use super::xml::rst_xml::{
    ds::{KeyInfo, KeyInfoTypeContent, XmlnsDsOpenEnumType},
    psf::{self, Pp, XmlnsPsfOpenEnumType},
    tns::{self, EndpointReference, Envelope, Fault, Header, XmlnsSOpenEnumType},
    wsp::{AppliesTo, XmlnsWsaOpenEnumType},
    wsse::{self, BinarySecurityToken, KeyIdentifier},
    wst::{
        BinarySecret, EncryptedDataTypeCipherData, EncryptedDataTypeEncryptionMethod, Lifetime,
        RequestSecurityTokenResponse, RequestSecurityTokenResponseCollection, RequestedProofToken,
        RequestedSecurityToken, RequestedSecurityTokenTypeEncryptedData, RequestedTokenReference,
        XmlnsOpenEnumType, XmlnsSamlOpenEnumType, XmlnsWspOpenEnumType, XmlnsWsseOpenEnumType,
        XmlnsWstOpenEnumType, XmlnsWsuOpenEnumType,
    },
    wsu::{Created, Expires},
    xs,
};
use argon2::password_hash::{
    SaltString,
    rand_core::{self, RngCore},
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::http::StatusCode;
use axum::{extract::State, response::IntoResponse};
use axum_serde::Xml;
use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use chrono::{Duration, Utc};
use log::{error, trace};
use quick_xml::events::{BytesDecl, Event};
use sqlx::{MySql, Pool};

enum ElementNotFoundError {
    Header,
    Body,
    Security,
    UsernameToken,
    RequestMultipleSecurityTokens,
    PolicyReference,
    Uri,
}

pub async fn rst(
    State(pool): State<Pool<MySql>>,
    Xml(envelope): Xml<Envelope>,
) -> impl IntoResponse {
    let Ok(username_token) = envelope
        .header
        .as_ref()
        .ok_or(ElementNotFoundError::Header)
        .and_then(|header| {
            header
                .security
                .as_ref()
                .ok_or(ElementNotFoundError::Security)
                .and_then(|security| {
                    security
                        .username_token
                        .as_ref()
                        .ok_or(ElementNotFoundError::UsernameToken)
                })
        })
    else {
        error!("Error getting username and password from XML");
        return invalid_request_envelope();
    };

    let Ok(user) = sqlx::query!(
        "SELECT id, email, password, puid FROM users WHERE email = ? LIMIT 1",
        username_token.username.content
    )
    .fetch_one(&pool)
    .await
    else {
        return invalid_request_envelope();
    };

    let parsed_hash = match PasswordHash::new(&user.password) {
        Ok(parsed_hash) => parsed_hash,
        Err(error) => {
            error!("Error hashing password: {error}");
            return failed_authentication_envelope();
        }
    };

    if let Err(error) =
        Argon2::default().verify_password(username_token.password.content.as_bytes(), &parsed_hash)
    {
        error!("Error verifying password: {error}");
        return failed_authentication_envelope();
    }

    let binary_secret = SaltString::generate(&mut rand_core::OsRng)
        .as_str()
        .to_string();

    let mut bytes = [0u8; 88];
    rand_core::OsRng.fill_bytes(&mut bytes);

    let mut generated_token = URL_SAFE.encode(bytes);
    generated_token.insert_str(0, "t=");

    let now = Utc::now().naive_utc();
    let datetime = now + Duration::hours(24);

    if sqlx::query!(
        "INSERT INTO tokens (token, valid_until, user_id) VALUES (?, ?, ?)",
        generated_token,
        datetime,
        user.id
    )
    .execute(&pool)
    .await
    .is_err()
    {
        return failed_authentication_envelope();
    }

    trace!("Generated token for {}", user.email);

    let Ok(request_multiple_security_tokens) = envelope
        .body
        .as_ref()
        .ok_or(ElementNotFoundError::Body)
        .and_then(|body| {
            body.request_multiple_security_tokens
                .as_ref()
                .ok_or(ElementNotFoundError::RequestMultipleSecurityTokens)
        })
    else {
        return invalid_request_envelope();
    };

    let mut request_security_token_response: Vec<RequestSecurityTokenResponse> = Vec::new();
    for security_token in &request_multiple_security_tokens.request_security_token {
        let Some(applies_to) = &security_token.applies_to else {
            return invalid_request_envelope();
        };

        let Some(security_token_id) = security_token.id.as_ref() else {
            return invalid_request_envelope();
        };

        let security_token_id = security_token_id.replace("RST", "");

        match applies_to.endpoint_reference.address.content.as_str() {
            "http://Passport.NET/tb" => {
                request_security_token_response.push(RequestSecurityTokenResponse {
                    token_type: Some("urn:passport:legacy".to_string()),
                    applies_to: Some(AppliesTo {
                        endpoint_reference: EndpointReference {
                            address: tns::AttributedQNameType {
                                content: applies_to.endpoint_reference.address.content.clone(),
                            },
                            reference_parameters: None,
                            metadata: None,
                        },
                        xmlns_wsa: Some(XmlnsWsaOpenEnumType::Addressing),
                    }),
                    lifetime: Some(Lifetime {
                        created: Some(Created {
                            id: None,
                            content: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                        }),
                        expires: Some(Expires {
                            id: None,
                            content: datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                        }),
                    }),
                    requested_security_token: Some(RequestedSecurityToken {
                        encrypted_data: Some(RequestedSecurityTokenTypeEncryptedData {
                            encryption_method: EncryptedDataTypeEncryptionMethod {
                                algorithm: Some(
                                    "http://www.w3.org/2001/04/xmlenc#tripledes-cbc".to_string(),
                                ),
                            },
                            id: Some("BinaryDAToken".to_string() + security_token_id.as_str()),
                            type_: Some("http://www.w3.org/2001/04/xmlenc#Element".to_string()),
                            key_info: KeyInfo {
                                id: None,
                                content: vec![KeyInfoTypeContent {
                                    key_name: Some("http://Passport.NET/STS".to_string()),
                                    key_value: None,
                                    retrieval_method: None,
                                    x509_data: None,
                                    pgp_data: None,
                                    spki_data: None,
                                    mgmt_data: None,
                                }],
                                xmlns_ds: Some(XmlnsDsOpenEnumType::XmldSig),
                            },
                            cipher_data: EncryptedDataTypeCipherData {
                                cipher_value: generated_token.replace("t=", ""),
                            },
                            xmlns: Some(XmlnsOpenEnumType::XmlEnc),
                        }),
                        binary_security_token: None,
                    }),
                    requested_attached_reference: None,
                    requested_unattached_reference: None,
                    requested_token_reference: Some(RequestedTokenReference {
                        key_identifier: KeyIdentifier {
                            value_type: Some("urn:passport".to_string()),
                        },
                        reference: wsse::Reference {
                            uri: Some("#BinaryDAToken".to_string() + security_token_id.as_str()),
                        },
                    }),
                    requested_proof_token: Some(RequestedProofToken {
                        binary_secret: BinarySecret {
                            content: binary_secret.clone(),
                        },
                    }),
                });
            }

            "messenger.msn.com" => {
                if let Ok(uri) = security_token
                    .policy_reference
                    .as_ref()
                    .ok_or(ElementNotFoundError::PolicyReference)
                    .and_then(|policy_reference| {
                        policy_reference
                            .uri
                            .as_ref()
                            .ok_or(ElementNotFoundError::Uri)
                    })
                {
                    if uri != "?ct=1&rver=1&wp=FS_40SEC_0_COMPACT&lc=1&id=1" {
                        return failed_authentication_envelope();
                    }
                } else {
                    return failed_authentication_envelope();
                }

                request_security_token_response.push(RequestSecurityTokenResponse {
                    token_type: Some("urn:passport:compat".to_string()),
                    applies_to: Some(AppliesTo {
                        endpoint_reference: EndpointReference {
                            address: tns::AttributedQNameType {
                                content: applies_to.endpoint_reference.address.content.clone(),
                            },
                            reference_parameters: None,
                            metadata: None,
                        },
                        xmlns_wsa: Some(XmlnsWsaOpenEnumType::Addressing),
                    }),
                    lifetime: Some(Lifetime {
                        created: Some(Created {
                            id: None,
                            content: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                        }),
                        expires: Some(Expires {
                            id: None,
                            content: datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                        }),
                    }),
                    requested_security_token: Some(RequestedSecurityToken {
                        encrypted_data: None,
                        binary_security_token: Some(BinarySecurityToken {
                            id: Some("Compact".to_string() + security_token_id.as_str()),
                            content: generated_token.clone(),
                        }),
                    }),
                    requested_attached_reference: None,
                    requested_unattached_reference: None,
                    requested_token_reference: Some(RequestedTokenReference {
                        key_identifier: KeyIdentifier {
                            value_type: Some("urn:passport:compact".to_string()),
                        },
                        reference: wsse::Reference {
                            uri: Some("#Compact".to_string() + security_token_id.as_str()),
                        },
                    }),
                    requested_proof_token: None,
                });
            }

            _ => return invalid_request_envelope(),
        }
    }

    let envelope = Envelope {
        header: Some(Header {
            pp: Some(Pp {
                server_version: 1,
                puid: user.puid.to_string(),
                config_version: "3.0.869.0".to_string(),
                ui_version: "3.0.869.0".to_string(),
                authstate: "0x48803".to_string(),
                reqstatus: "0x0".to_string(),
                server_info: psf::ServerInfo {
                    path: Some("Live1".to_string()),
                    server_time: Some(
                        Utc::now()
                            .naive_utc()
                            .format("%Y-%m-%dT%H:%M:%SZ")
                            .to_string(),
                    ),
                    loc_version: Some(0),
                    rolling_upgrade_state: Some("ExclusiveNew".to_string()),
                    content: "BAYPPLOGN3B12 2006.01.27.13.57.29".to_string(),
                },
                cookies: xs::AnyType,
                browser_cookies: None,
                cred_properties: None,
                ext_properties: None,
                response: xs::AnyType,
                xmlns_psf: Some(XmlnsPsfOpenEnumType::SoapServicesSoapFault),
            }),
            authinfo: None,
            security: None,
        }),
        body: Some(tns::Body {
            request_security_token_response_collection: Some(
                RequestSecurityTokenResponseCollection {
                    request_security_token_response,
                    xmlns_s: Some(XmlnsSOpenEnumType::SoapEnvelope),
                    xmlns_wst: Some(XmlnsWstOpenEnumType::Trust),
                    xmlns_wsse: Some(XmlnsWsseOpenEnumType::Secext),
                    xmlns_wsu: Some(XmlnsWsuOpenEnumType::WssSecurity),
                    xmlns_saml: Some(XmlnsSamlOpenEnumType::UrnAssertion),
                    xmlns_wsp: Some(XmlnsWspOpenEnumType::Policy),
                    xmlns_psf: Some(XmlnsPsfOpenEnumType::SoapServicesSoapFault),
                },
            ),
            request_multiple_security_tokens: None,
        }),
        fault: None,
        xmlns_s: Some(XmlnsSOpenEnumType::SoapEnvelope),
    };

    let mut buffer = Vec::new();
    let mut writer = quick_xml::Writer::new_with_indent(&mut buffer, b' ', 4);

    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    writer
        .write_serializable("S:Envelope", &envelope)
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    trace!("Serialized RST response for {}", user.email);
    String::from_utf8(buffer).or(Err(StatusCode::INTERNAL_SERVER_ERROR))
}

fn invalid_request_envelope() -> Result<String, StatusCode> {
    let envelope = Envelope {
        fault: Some(Fault {
            faultcode: "S:Client".to_string(),
            faultstring: "Invalid Request".to_string(),
            faultactor: None,
            detail: None,
        }),
        header: None,
        body: None,
        xmlns_s: Some(XmlnsSOpenEnumType::SoapEnvelope),
    };

    let mut buffer = Vec::new();
    let mut writer = quick_xml::Writer::new_with_indent(&mut buffer, b' ', 4);

    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    writer
        .write_serializable("S:Envelope", &envelope)
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    String::from_utf8(buffer).or(Err(StatusCode::INTERNAL_SERVER_ERROR))
}

fn failed_authentication_envelope() -> Result<String, StatusCode> {
    let envelope = Envelope {
        header: Some(Header {
            pp: Some(Pp {
                server_version: 1,
                puid: "".to_string(),
                config_version: "3.0.869.0".to_string(),
                ui_version: "3.0.869.0".to_string(),
                authstate: "0x48803".to_string(),
                reqstatus: "0x0".to_string(),
                server_info: psf::ServerInfo {
                    path: Some("Live1".to_string()),
                    server_time: Some(
                        Utc::now()
                            .naive_utc()
                            .format("%Y-%m-%dT%H:%M:%SZ")
                            .to_string(),
                    ),
                    loc_version: Some(0),
                    rolling_upgrade_state: Some("ExclusiveNew".to_string()),
                    content: "BAYPPLOGN3B12 2006.01.27.13.57.29".to_string(),
                },
                cookies: xs::AnyType,
                browser_cookies: None,
                cred_properties: None,
                ext_properties: None,
                response: xs::AnyType,
                xmlns_psf: Some(XmlnsPsfOpenEnumType::SoapServicesSoapFault),
            }),
            authinfo: None,
            security: None,
        }),
        fault: Some(Fault {
            faultcode: "wsse:FailedAuthentication".to_string(),
            faultstring: "Authentication Failure".to_string(),
            faultactor: None,
            detail: None,
        }),
        body: None,
        xmlns_s: Some(XmlnsSOpenEnumType::SoapEnvelope),
    };

    let mut buffer = Vec::new();
    let mut writer = quick_xml::Writer::new_with_indent(&mut buffer, b' ', 4);

    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    writer
        .write_serializable("S:Envelope", &envelope)
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    String::from_utf8(buffer).or(Err(StatusCode::INTERNAL_SERVER_ERROR))
}
