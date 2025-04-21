#[allow(dead_code)]
pub mod tns {
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Serialize, Deserialize)]
    pub struct BodyType {
        #[serde(
            rename = "RequestMultipleSecurityTokens",
            skip_serializing_if = "Option::is_none"
        )]
        pub request_multiple_security_tokens: Option<super::ps::RequestMultipleSecurityTokens>,
        #[serde(
            rename(
                serialize = "wst:RequestSecurityTokenResponseCollection",
                deserialize = "RequestSecurityTokenResponseCollection"
            ),
            skip_serializing_if = "Option::is_none"
        )]
        pub request_security_token_response_collection:
            Option<super::wst::RequestSecurityTokenResponseCollection>,
    }
    pub type Body = BodyType;
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename(serialize = "S:Envelope", deserialize = "Envelope"))]
    pub struct EnvelopeType {
        #[serde(rename = "@xmlns:S", skip_serializing_if = "Option::is_none")]
        pub xmlns_s: Option<XmlnsSOpenEnumType>,
        #[serde(
            default,
            rename(serialize = "S:Header", deserialize = "Header"),
            skip_serializing_if = "Option::is_none"
        )]
        pub header: Option<HeaderType>,
        #[serde(
            rename(serialize = "S:Body", deserialize = "Body"),
            skip_serializing_if = "Option::is_none"
        )]
        pub body: Option<BodyType>,
        #[serde(
            rename(serialize = "S:Fault", deserialize = "Fault"),
            skip_serializing_if = "Option::is_none"
        )]
        pub fault: Option<FaultType>,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub enum XmlnsSOpenEnumType {
        #[serde(rename = "http://schemas.xmlsoap.org/soap/envelope/")]
        SoapEnvelope,
    }
    pub type Envelope = EnvelopeType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct FaultType {
        #[serde(rename = "faultcode")]
        pub faultcode: String,
        #[serde(rename = "faultstring")]
        pub faultstring: String,
        #[serde(
            default,
            rename = "faultactor",
            skip_serializing_if = "Option::is_none"
        )]
        pub faultactor: Option<String>,
        #[serde(default, rename = "detail", skip_serializing_if = "Option::is_none")]
        pub detail: Option<BodyType>,
    }
    pub type Fault = FaultType;
    pub type FaultDetail = BodyType;
    pub type FaultFaultactor = String;
    pub type FaultFaultcode = String;
    pub type FaultFaultstring = String;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct HeaderType {
        #[serde(
            rename(serialize = "psf:pp", deserialize = "pp"),
            skip_serializing_if = "Option::is_none"
        )]
        pub pp: Option<super::psf::Pp>,
        #[serde(rename = "AuthInfo", skip_serializing_if = "Option::is_none")]
        pub authinfo: Option<super::ps::AuthInfo>,
        #[serde(rename = "Security", skip_serializing_if = "Option::is_none")]
        pub security: Option<super::wsse::Security>,
    }
    pub type Header = HeaderType;
    pub type Actor = String;
    pub type DetailType = BodyType;
    pub type EncodingStyleType = super::xs::EntitiesType;
    pub type EncodingStyle = super::xs::EntitiesType;
    pub type MustUnderstand = bool;
    pub type Action = ActionElementType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ActionElementType {
        #[serde(rename = "$text")]
        pub content: String,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub struct AttributedQNameType {
        #[serde(rename = "$text")]
        pub content: String,
    }
    pub type AttributedUriType = AttributedQNameType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct AttributedUnsignedLongType {
        #[serde(rename = "$text")]
        pub content: u64,
    }
    pub type EndpointReference = EndpointReferenceType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct EndpointReferenceType {
        #[serde(rename(serialize = "wsa:Address", deserialize = "Address"))]
        pub address: AttributedQNameType,
        #[serde(
            default,
            rename = "ReferenceParameters",
            skip_serializing_if = "Option::is_none"
        )]
        pub reference_parameters: Option<MetadataType>,
        #[serde(default, rename = "Metadata", skip_serializing_if = "Option::is_none")]
        pub metadata: Option<MetadataType>,
    }
    pub type EndpointReferenceTypeAddress = AttributedQNameType;
    #[derive(Debug, Serialize, Deserialize)]
    pub enum FaultCodesOpenEnumType {
        #[serde(rename = "tns:InvalidAddressingHeader")]
        TnsInvalidAddressingHeader,
        #[serde(rename = "tns:InvalidAddress")]
        TnsInvalidAddress,
        #[serde(rename = "tns:InvalidEPR")]
        TnsInvalidEpr,
        #[serde(rename = "tns:InvalidCardinality")]
        TnsInvalidCardinality,
        #[serde(rename = "tns:MissingAddressInEPR")]
        TnsMissingAddressInEpr,
        #[serde(rename = "tns:DuplicateMessageID")]
        TnsDuplicateMessageId,
        #[serde(rename = "tns:ActionMismatch")]
        TnsActionMismatch,
        #[serde(rename = "tns:MessageAddressingHeaderRequired")]
        TnsMessageAddressingHeaderRequired,
        #[serde(rename = "tns:DestinationUnreachable")]
        TnsDestinationUnreachable,
        #[serde(rename = "tns:ActionNotSupported")]
        TnsActionNotSupported,
        #[serde(rename = "tns:EndpointUnavailable")]
        TnsEndpointUnavailable,
        #[serde(untagged)]
        String(String),
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub enum FaultCodesType {
        #[serde(rename = "tns:InvalidAddressingHeader")]
        TnsInvalidAddressingHeader,
        #[serde(rename = "tns:InvalidAddress")]
        TnsInvalidAddress,
        #[serde(rename = "tns:InvalidEPR")]
        TnsInvalidEpr,
        #[serde(rename = "tns:InvalidCardinality")]
        TnsInvalidCardinality,
        #[serde(rename = "tns:MissingAddressInEPR")]
        TnsMissingAddressInEpr,
        #[serde(rename = "tns:DuplicateMessageID")]
        TnsDuplicateMessageId,
        #[serde(rename = "tns:ActionMismatch")]
        TnsActionMismatch,
        #[serde(rename = "tns:MessageAddressingHeaderRequired")]
        TnsMessageAddressingHeaderRequired,
        #[serde(rename = "tns:DestinationUnreachable")]
        TnsDestinationUnreachable,
        #[serde(rename = "tns:ActionNotSupported")]
        TnsActionNotSupported,
        #[serde(rename = "tns:EndpointUnavailable")]
        TnsEndpointUnavailable,
    }
    pub type FaultTo = EndpointReferenceType;
    pub type From = EndpointReferenceType;
    pub type IsReferenceParameter = bool;
    pub type MessageId = AttributedQNameType;
    pub type Metadata = MetadataType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct MetadataType;
    pub type ProblemAction = ProblemActionType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ProblemActionType {
        #[serde(default, rename = "Action")]
        pub action: Option<ActionElementType>,
        #[serde(default, rename = "SoapAction")]
        pub soap_action: Option<String>,
    }
    pub type ProblemActionTypeSoapAction = String;
    pub type ProblemHeaderQName = AttributedQNameType;
    pub type ProblemIri = AttributedQNameType;
    pub type ReferenceParameters = MetadataType;
    pub type ReferenceParametersType = MetadataType;
    pub type RelatesTo = RelatesToType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct RelatesToType {
        #[serde(
            //default = "RelatesToType::default_relationship_type",
            rename = "@RelationshipType"
        )]
        pub relationship_type: RelationshipTypeOpenEnumType,
        #[serde(rename = "$text")]
        pub content: String,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub enum RelationshipType {
        #[serde(rename = "http://www.w3.org/2005/08/addressing/reply")]
        HttpWwwW3Org200508AddressingReply,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub enum RelationshipTypeOpenEnumType {
        #[serde(rename = "http://www.w3.org/2005/08/addressing/reply")]
        HttpWwwW3Org200508AddressingReply,
        #[serde(untagged)]
        String(String),
    }
    pub type ReplyTo = EndpointReferenceType;
    pub type RetryAfter = AttributedUnsignedLongType;
    pub type To = ActionElementType;
    pub type ToElementType = ActionElementType;
}

#[allow(dead_code)]
pub mod xs {
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Serialize, Deserialize, Default)]
    pub struct EntitiesType(pub Vec<String>);
    pub type EntityType = EntitiesType;
    pub type IdType = String;
    pub type IdrefType = String;
    pub type IdrefsType = EntitiesType;
    pub type NcNameType = String;
    pub type NmtokenType = String;
    pub type NmtokensType = EntitiesType;
    pub type NotationType = String;
    pub type NameType = String;
    pub type QnameType = String;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct AnyType;
    pub type AnyUriType = String;
    pub type Base64BinaryType = String;
    pub type BooleanType = bool;
    pub type ByteType = i8;
    pub type DateType = String;
    pub type DateTimeType = String;
    pub type DecimalType = f64;
    pub type DoubleType = f64;
    pub type DurationType = String;
    pub type FloatType = f32;
    pub type GdayType = String;
    pub type GmonthType = String;
    pub type GmonthDayType = String;
    pub type GyearType = String;
    pub type GyearMonthType = String;
    pub type HexBinaryType = String;
    pub type IntType = i32;
    pub type IntegerType = i32;
    pub type LanguageType = String;
    pub type LongType = i64;
    pub type NegativeIntegerType = isize;
    pub type NonNegativeIntegerType = usize;
    pub type NonPositiveIntegerType = isize;
    pub type NormalizedStringType = String;
    pub type PositiveIntegerType = usize;
    pub type ShortType = i16;
    pub type StringType = String;
    pub type TimeType = String;
    pub type TokenType = String;
    pub type UnsignedByteType = u8;
    pub type UnsignedIntType = u32;
    pub type UnsignedLongType = u64;
    pub type UnsignedShortType = u16;
}

#[allow(dead_code)]
pub mod ds {
    use serde::{Deserialize, Serialize};
    pub type CanonicalizationMethod = CanonicalizationMethodType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct CanonicalizationMethodType {
        #[serde(rename = "@Algorithm")]
        pub algorithm: String,
    }
    pub type CryptoBinaryType = String;
    pub type DsaKeyValue = DsaKeyValueType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct DsaKeyValueType {
        #[serde(default, rename = "P")]
        pub p: Option<String>,
        #[serde(default, rename = "Q")]
        pub q: Option<String>,
        #[serde(default, rename = "G")]
        pub g: Option<String>,
        #[serde(rename = "Y")]
        pub y: String,
        #[serde(default, rename = "J")]
        pub j: Option<String>,
        #[serde(default, rename = "Seed")]
        pub seed: Option<String>,
        #[serde(default, rename = "PgenCounter")]
        pub pgen_counter: Option<String>,
    }
    pub type DigestMethod = DigestMethodType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct DigestMethodType {
        #[serde(rename = "@Algorithm")]
        pub algorithm: String,
    }
    pub type DigestValue = String;
    pub type DigestValueType = String;
    pub type DsaKeyValueTypeG = String;
    pub type DsaKeyValueTypeJ = String;
    pub type DsaKeyValueTypeP = String;
    pub type DsaKeyValueTypePgenCounter = String;
    pub type DsaKeyValueTypeQ = String;
    pub type DsaKeyValueTypeSeed = String;
    pub type DsaKeyValueTypeY = String;
    pub type HmacOutputLengthType = i32;
    pub type KeyInfo = KeyInfoType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct KeyInfoType {
        #[serde(rename = "@xmlns:ds", skip_serializing_if = "Option::is_none")]
        pub xmlns_ds: Option<XmlnsDsOpenEnumType>,
        #[serde(default, rename = "@Id", skip_serializing_if = "Option::is_none")]
        pub id: Option<String>,
        #[serde(rename = "KeyInfo")]
        pub content: Vec<KeyInfoTypeContent>,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub enum XmlnsDsOpenEnumType {
        #[serde(rename = "http://www.w3.org/2000/09/xmldsig#")]
        XmldSig,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub struct KeyInfoTypeContent {
        #[serde(default, rename = "KeyName", skip_serializing_if = "Option::is_none")]
        pub key_name: Option<String>,
        #[serde(default, rename = "KeyValue", skip_serializing_if = "Option::is_none")]
        pub key_value: Option<KeyValueType>,
        #[serde(
            default,
            rename = "RetrievalMethod",
            skip_serializing_if = "Option::is_none"
        )]
        pub retrieval_method: Option<RetrievalMethodType>,
        #[serde(default, rename = "X509Data", skip_serializing_if = "Option::is_none")]
        pub x509_data: Option<X509DataType>,
        #[serde(default, rename = "PGPData", skip_serializing_if = "Option::is_none")]
        pub pgp_data: Option<PgpDataType>,
        #[serde(default, rename = "SPKIData", skip_serializing_if = "Option::is_none")]
        pub spki_data: Option<SpkiDataType>,
        #[serde(default, rename = "MgmtData", skip_serializing_if = "Option::is_none")]
        pub mgmt_data: Option<String>,
    }
    pub type KeyName = String;
    pub type KeyValue = KeyValueType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct KeyValueType {
        #[serde(default, rename = "DSAKeyValue")]
        pub dsa_key_value: Option<DsaKeyValueType>,
        #[serde(default, rename = "RSAKeyValue")]
        pub rsa_key_value: Option<RsaKeyValueType>,
    }
    pub type Manifest = ManifestType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ManifestType {
        #[serde(default, rename = "@Id")]
        pub id: Option<String>,
        #[serde(default, rename = "Reference")]
        pub reference: Vec<ReferenceType>,
    }
    pub type MgmtData = String;
    pub type Object = ObjectType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ObjectType {
        #[serde(default, rename = "@Id")]
        pub id: Option<String>,
        #[serde(default, rename = "@MimeType")]
        pub mime_type: Option<String>,
        #[serde(default, rename = "@Encoding")]
        pub encoding: Option<String>,
    }
    pub type PgpData = PgpDataType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct PgpDataType {
        #[serde(default, rename = "PGPKeyID")]
        pub pgp_key_id: Option<String>,
        #[serde(default, rename = "PGPKeyPacket")]
        pub pgp_key_packet: Option<String>,
    }
    pub type PgpDataTypePgpKeyId = String;
    pub type PgpDataTypePgpKeyPacket = String;
    pub type RsaKeyValue = RsaKeyValueType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct RsaKeyValueType {
        #[serde(rename = "Modulus")]
        pub modulus: String,
        #[serde(rename = "Exponent")]
        pub exponent: String,
    }
    pub type Reference = ReferenceType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ReferenceType {
        #[serde(default, rename = "@Id")]
        pub id: Option<String>,
        #[serde(default, rename = "@URI")]
        pub uri: Option<String>,
        #[serde(default, rename = "@Type")]
        pub type_: Option<String>,
        #[serde(default, rename = "Transforms")]
        pub transforms: Option<TransformsType>,
        #[serde(rename = "DigestMethod")]
        pub digest_method: DigestMethodType,
        #[serde(rename = "DigestValue")]
        pub digest_value: String,
    }
    pub type RetrievalMethod = RetrievalMethodType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct RetrievalMethodType {
        #[serde(default, rename = "@URI")]
        pub uri: Option<String>,
        #[serde(default, rename = "@Type")]
        pub type_: Option<String>,
        #[serde(default, rename = "Transforms")]
        pub transforms: Option<TransformsType>,
    }
    pub type RsaKeyValueTypeExponent = String;
    pub type RsaKeyValueTypeModulus = String;
    pub type SpkiData = SpkiDataType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SpkiDataType {
        #[serde(rename = "$value")]
        pub content: Vec<SpkiDataTypeContent>,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SpkiDataTypeContent {
        #[serde(rename = "SPKISexp")]
        pub spki_sexp: String,
    }
    pub type Signature = SignatureType;
    pub type SignatureMethod = SignatureMethodType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SignatureMethodType {
        #[serde(rename = "@Algorithm")]
        pub algorithm: String,
        #[serde(default, rename = "HMACOutputLength")]
        pub hmac_output_length: Option<i32>,
    }
    pub type SignatureMethodTypeHmacOutputLength = i32;
    pub type SignatureProperties = SignaturePropertiesType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SignaturePropertiesType {
        #[serde(default, rename = "@Id")]
        pub id: Option<String>,
        #[serde(default, rename = "SignatureProperty")]
        pub signature_property: Vec<SignaturePropertyType>,
    }
    pub type SignatureProperty = SignaturePropertyType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SignaturePropertyType {
        #[serde(rename = "@Target")]
        pub target: String,
        #[serde(default, rename = "@Id")]
        pub id: Option<String>,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SignatureType {
        #[serde(default, rename = "@Id")]
        pub id: Option<String>,
        #[serde(rename = "SignedInfo")]
        pub signed_info: SignedInfoType,
        #[serde(rename = "SignatureValue")]
        pub signature_value: SignatureValueType,
        #[serde(default, rename = "KeyInfo")]
        pub key_info: Option<KeyInfoType>,
        #[serde(default, rename = "Object")]
        pub object: Vec<ObjectType>,
    }
    pub type SignatureValue = SignatureValueType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SignatureValueType {
        #[serde(default, rename = "@Id")]
        pub id: Option<String>,
        #[serde(rename = "$text")]
        pub content: String,
    }
    pub type SignedInfo = SignedInfoType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SignedInfoType {
        #[serde(default, rename = "@Id")]
        pub id: Option<String>,
        #[serde(rename = "CanonicalizationMethod")]
        pub canonicalization_method: CanonicalizationMethodType,
        #[serde(rename = "SignatureMethod")]
        pub signature_method: SignatureMethodType,
        #[serde(default, rename = "Reference")]
        pub reference: Vec<ReferenceType>,
    }
    pub type SpkiDataTypeSpkiSexp = String;
    pub type Transform = TransformType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct TransformType {
        #[serde(rename = "@Algorithm")]
        pub algorithm: String,
        #[serde(default, rename = "$value")]
        pub content: Vec<TransformTypeContent>,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub struct TransformTypeContent {
        #[serde(default, rename = "XPath")]
        pub x_path: Option<String>,
    }
    pub type TransformTypeXPath = String;
    pub type Transforms = TransformsType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct TransformsType {
        #[serde(default, rename = "Transform")]
        pub transform: Vec<TransformType>,
    }
    pub type X509Data = X509DataType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct X509DataType {
        #[serde(rename = "$value")]
        pub content: Vec<X509DataTypeContent>,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub struct X509DataTypeContent {
        #[serde(default, rename = "X509IssuerSerial")]
        pub x509_issuer_serial: Option<X509IssuerSerialType>,
        #[serde(default, rename = "X509SKI")]
        pub x509_ski: Option<String>,
        #[serde(default, rename = "X509SubjectName")]
        pub x509_subject_name: Option<String>,
        #[serde(default, rename = "X509Certificate")]
        pub x509_certificate: Option<String>,
        #[serde(default, rename = "X509CRL")]
        pub x509_crl: Option<String>,
    }
    pub type X509DataTypeX509Certificate = String;
    pub type X509DataTypeX509Crl = String;
    pub type X509DataTypeX509IssuerSerial = X509IssuerSerialType;
    pub type X509DataTypeX509Ski = String;
    pub type X509DataTypeX509SubjectName = String;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct X509IssuerSerialType {
        #[serde(rename = "X509IssuerName")]
        pub x509_issuer_name: String,
        #[serde(rename = "X509SerialNumber")]
        pub x509_serial_number: i32,
    }
    pub type X509IssuerSerialTypeX509IssuerName = String;
    pub type X509IssuerSerialTypeX509SerialNumber = i32;
}

#[allow(dead_code)]
pub mod ps {
    use serde::{Deserialize, Serialize};
    pub type AuthInfo = AuthInfoType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct AuthInfoType {
        #[serde(default, rename = "@Id")]
        pub id: Option<String>,
        #[serde(rename = "HostingApp")]
        pub hosting_app: String,
        #[serde(rename = "BinaryVersion")]
        pub binary_version: String,
        #[serde(rename = "UIVersion")]
        pub ui_version: String,
        #[serde(rename = "Cookies")]
        pub cookies: String,
        #[serde(rename = "RequestParams")]
        pub request_params: String,
    }
    pub type AuthInfoTypeBinaryVersion = String;
    pub type AuthInfoTypeCookies = String;
    pub type AuthInfoTypeHostingApp = String;
    pub type AuthInfoTypeRequestParams = String;
    pub type AuthInfoTypeUiVersion = String;
    pub type Id = String;
    pub type RequestMultipleSecurityTokens = RequestMultipleSecurityTokensType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct RequestMultipleSecurityTokensType {
        #[serde(default, rename = "@Id")]
        pub id: Option<String>,
        #[serde(default, rename = "RequestSecurityToken")]
        pub request_security_token: Vec<super::wst::RequestSecurityTokenType>,
    }
}

#[allow(dead_code)]
pub mod wsp {
    use serde::{Deserialize, Serialize};
    pub type AppliesTo = AppliesToElementType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct AppliesToElementType {
        #[serde(rename = "@xmlns:wsa", skip_serializing_if = "Option::is_none")]
        pub xmlns_wsa: Option<XmlnsWsaOpenEnumType>,
        #[serde(rename(serialize = "wsa:EndpointReference", deserialize = "EndpointReference"))]
        pub endpoint_reference: super::tns::EndpointReferenceType,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub enum XmlnsWsaOpenEnumType {
        #[serde(rename = "http://schemas.xmlsoap.org/ws/2004/03/addressing")]
        Addressing,
    }
    pub type PolicyReference = PolicyReferenceElementType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct PolicyReferenceElementType {
        #[serde(default, rename = "@URI")]
        pub uri: Option<String>,
    }
}

#[allow(dead_code)]
pub mod wsse {
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Serialize, Deserialize)]
    pub struct AttributedStringType {
        #[serde(default, rename = "@Id")]
        pub id: Option<String>,
        #[serde(rename = "$text")]
        pub content: String,
    }
    pub type BinarySecurityToken = BinarySecurityTokenType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct BinarySecurityTokenType {
        #[serde(default, rename = "@Id")]
        pub id: Option<String>,
        #[serde(rename = "$text")]
        pub content: String,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub struct EncodedStringType {
        #[serde(default, rename = "@Id")]
        pub id: Option<String>,
        #[serde(rename = "$text")]
        pub content: String,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub enum FaultcodeEnumType {
        #[serde(rename = "wsse:UnsupportedSecurityToken")]
        WsseUnsupportedSecurityToken,
        #[serde(rename = "wsse:UnsupportedAlgorithm")]
        WsseUnsupportedAlgorithm,
        #[serde(rename = "wsse:InvalidSecurity")]
        WsseInvalidSecurity,
        #[serde(rename = "wsse:InvalidSecurityToken")]
        WsseInvalidSecurityToken,
        #[serde(rename = "wsse:FailedAuthentication")]
        WsseFailedAuthentication,
        #[serde(rename = "wsse:FailedCheck")]
        WsseFailedCheck,
        #[serde(rename = "wsse:SecurityTokenUnavailable")]
        WsseSecurityTokenUnavailable,
    }
    pub type Id = String;
    pub type KeyIdentifier = KeyIdentifierType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct KeyIdentifierType {
        #[serde(default, rename = "@ValueType")]
        pub value_type: Option<String>,
    }
    pub type PasswordStringType = EncodedStringType;
    pub type PolicyReference = PolicyReferenceElementType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct PolicyReferenceElementType {
        #[serde(rename = "@URI")]
        pub uri: String,
    }
    pub type Reference = ReferenceType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ReferenceType {
        #[serde(default, rename = "@URI")]
        pub uri: Option<String>,
    }
    pub type Security = SecurityHeaderType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SecurityHeaderType {
        #[serde(default, rename = "UsernameToken")]
        pub username_token: Option<UsernameTokenType>,
        #[serde(default, rename = "Timestamp")]
        pub timestamp: Option<super::wsu::TimestampType>,
    }
    pub type SecurityHeaderTypeUsernameToken = UsernameTokenType;
    pub type SecurityTokenReference = SecurityTokenReferenceType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SecurityTokenReferenceType {
        #[serde(default, rename = "@Id")]
        pub id: Option<String>,
        #[serde(default, rename = "@Usage")]
        pub usage: Option<String>,
        #[serde(default, rename = "$value")]
        pub content: Vec<SecurityTokenReferenceTypeContent>,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SecurityTokenReferenceTypeContent {
        #[serde(default, rename = "Reference")]
        pub reference: Option<ReferenceType>,
    }
    pub type Usage = String;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct UsernameTokenType {
        #[serde(default, rename = "@Id")]
        pub id: Option<String>,
        #[serde(rename = "Username")]
        pub username: AttributedStringType,
        #[serde(rename = "Password")]
        pub password: EncodedStringType,
    }
    pub type UsernameTokenTypePassword = EncodedStringType;
    pub type UsernameTokenTypeUsername = AttributedStringType;
}

#[allow(dead_code)]
pub mod wst {
    use serde::{Deserialize, Serialize};
    pub type BinarySecret = BinarySecretType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct BinarySecretType {
        #[serde(rename = "$text")]
        pub content: String,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub enum BinarySecretTypeEnumType {
        #[serde(rename = "http://schemas.xmlsoap.org/ws/2005/02/trust/AsymmetricKey")]
        HttpSchemasXmlsoapOrgWs200502TrustAsymmetricKey,
        #[serde(rename = "http://schemas.xmlsoap.org/ws/2005/02/trust/SymmetricKey")]
        HttpSchemasXmlsoapOrgWs200502TrustSymmetricKey,
        #[serde(rename = "http://schemas.xmlsoap.org/ws/2005/02/trust/Nonce")]
        HttpSchemasXmlsoapOrgWs200502TrustNonce,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub enum BinarySecretTypeOpenEnumType {
        #[serde(rename = "http://schemas.xmlsoap.org/ws/2005/02/trust/AsymmetricKey")]
        HttpSchemasXmlsoapOrgWs200502TrustAsymmetricKey,
        #[serde(rename = "http://schemas.xmlsoap.org/ws/2005/02/trust/SymmetricKey")]
        HttpSchemasXmlsoapOrgWs200502TrustSymmetricKey,
        #[serde(rename = "http://schemas.xmlsoap.org/ws/2005/02/trust/Nonce")]
        HttpSchemasXmlsoapOrgWs200502TrustNonce,
        #[serde(untagged)]
        String(String),
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub struct CipherDataType {
        #[serde(rename = "CipherValue")]
        pub cipher_value: String,
    }
    pub type CipherDataTypeCipherValue = String;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct EncryptedDataType {
        #[serde(rename = "@xmlns", skip_serializing_if = "Option::is_none")]
        pub xmlns: Option<XmlnsOpenEnumType>,
        #[serde(default, rename = "@Id")]
        pub id: Option<String>,
        #[serde(default, rename = "@Type")]
        pub type_: Option<String>,
        #[serde(rename = "EncryptionMethod")]
        pub encryption_method: EncryptionMethodType,
        #[serde(rename(serialize = "ds:KeyInfo", deserialize = "KeyInfo"))]
        pub key_info: super::ds::KeyInfoType,
        #[serde(rename = "CipherData")]
        pub cipher_data: CipherDataType,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub enum XmlnsOpenEnumType {
        #[serde(rename = "http://www.w3.org/2001/04/xmlenc#")]
        XmlEnc,
    }
    pub type EncryptedDataTypeCipherData = CipherDataType;
    pub type EncryptedDataTypeEncryptionMethod = EncryptionMethodType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct EncryptionMethodType {
        #[serde(default, rename = "@Algorithm")]
        pub algorithm: Option<String>,
    }
    pub type Id = String;
    pub type Lifetime = LifetimeType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct LifetimeType {
        #[serde(default, rename(serialize = "wsu:Created", deserialize = "Created"))]
        pub created: Option<super::wsu::AttributedDateTimeType>,
        #[serde(default, rename(serialize = "wsu:Expires", deserialize = "Expires"))]
        pub expires: Option<super::wsu::AttributedDateTimeType>,
    }
    pub type RequestSecurityToken = RequestSecurityTokenType;
    pub type RequestSecurityTokenResponse = RequestSecurityTokenResponseType;
    pub type RequestSecurityTokenResponseCollection = RequestSecurityTokenResponseCollectionType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct RequestSecurityTokenResponseCollectionType {
        #[serde(rename = "@xmlns:S", skip_serializing_if = "Option::is_none")]
        pub xmlns_s: Option<super::tns::XmlnsSOpenEnumType>,
        #[serde(rename = "@xmlns:wst", skip_serializing_if = "Option::is_none")]
        pub xmlns_wst: Option<XmlnsWstOpenEnumType>,
        #[serde(rename = "@xmlns:wsse", skip_serializing_if = "Option::is_none")]
        pub xmlns_wsse: Option<XmlnsWsseOpenEnumType>,
        #[serde(rename = "@xmlns:wsu", skip_serializing_if = "Option::is_none")]
        pub xmlns_wsu: Option<XmlnsWsuOpenEnumType>,
        #[serde(rename = "@xmlns:saml", skip_serializing_if = "Option::is_none")]
        pub xmlns_saml: Option<XmlnsSamlOpenEnumType>,
        #[serde(rename = "@xmlns:wsp", skip_serializing_if = "Option::is_none")]
        pub xmlns_wsp: Option<XmlnsWspOpenEnumType>,
        #[serde(rename = "@xmlns:psf", skip_serializing_if = "Option::is_none")]
        pub xmlns_psf: Option<super::psf::XmlnsPsfOpenEnumType>,
        #[serde(
            default,
            rename(
                serialize = "wst:RequestSecurityTokenResponse",
                deserialize = "RequestSecurityTokenResponse"
            )
        )]
        pub request_security_token_response: Vec<RequestSecurityTokenResponseType>,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub enum XmlnsWstOpenEnumType {
        #[serde(rename = "http://schemas.xmlsoap.org/ws/2004/04/trust")]
        Trust,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub enum XmlnsWsseOpenEnumType {
        #[serde(rename = "http://schemas.xmlsoap.org/ws/2003/06/secext")]
        Secext,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub enum XmlnsWsuOpenEnumType {
        #[serde(
            rename = "http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd"
        )]
        WssSecurity,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub enum XmlnsSamlOpenEnumType {
        #[serde(rename = "urn:oasis:names:tc:SAML:1.0:assertion")]
        UrnAssertion,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub enum XmlnsWspOpenEnumType {
        #[serde(rename = "http://schemas.xmlsoap.org/ws/2002/12/policy")]
        Policy,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub struct RequestSecurityTokenResponseType {
        #[serde(
            default,
            rename(serialize = "wst:TokenType", deserialize = "TokenType")
        )]
        pub token_type: Option<String>,
        #[serde(
            default,
            rename(serialize = "wsp:AppliesTo", deserialize = "AppliesTo")
        )]
        pub applies_to: Option<super::wsp::AppliesToElementType>,
        #[serde(default, rename(serialize = "wst:Lifetime", deserialize = "Lifetime"))]
        pub lifetime: Option<LifetimeType>,
        #[serde(
            default,
            rename(
                serialize = "wst:RequestedSecurityToken",
                deserialize = "RequestedSecurityToken"
            ),
            skip_serializing_if = "Option::is_none"
        )]
        pub requested_security_token: Option<RequestedSecurityTokenType>,
        #[serde(
            default,
            rename(
                serialize = "wst:RequestedAttachedReference",
                deserialize = "RequestedAttachedReference"
            ),
            skip_serializing_if = "Option::is_none"
        )]
        pub requested_attached_reference: Option<RequestedAttachedReferenceElementType>,
        #[serde(
            default,
            rename(
                serialize = "wst:RequestedUnattachedReference",
                deserialize = "RequestedUnattachedReference"
            ),
            skip_serializing_if = "Option::is_none"
        )]
        pub requested_unattached_reference: Option<RequestedAttachedReferenceElementType>,
        #[serde(
            default,
            rename(
                serialize = "wst:RequestedTokenReference",
                deserialize = "RequestedTokenReference"
            ),
            skip_serializing_if = "Option::is_none"
        )]
        pub requested_token_reference: Option<RequestedTokenReferenceType>,
        #[serde(
            default,
            rename(
                serialize = "wst:RequestedProofToken",
                deserialize = "RequestedProofToken"
            ),
            skip_serializing_if = "Option::is_none"
        )]
        pub requested_proof_token: Option<RequestedProofTokenType>,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub struct RequestSecurityTokenType {
        #[serde(default, rename = "@Id")]
        pub id: Option<String>,
        #[serde(default, rename = "TokenType")]
        pub token_type: Option<String>,
        #[serde(rename = "RequestType")]
        pub request_type: RequestTypeEnumType,
        #[serde(default, rename = "AppliesTo")]
        pub applies_to: Option<super::wsp::AppliesToElementType>,
        #[serde(default, rename = "PolicyReference")]
        pub policy_reference: Option<super::wsp::PolicyReferenceElementType>,
    }
    pub type RequestType = RequestTypeEnumType;
    #[derive(Debug, Serialize, Deserialize)]
    pub enum RequestTypeEnumType {
        #[serde(rename = "http://schemas.xmlsoap.org/ws/2005/02/trust/Issue")]
        HttpSchemasXmlsoapOrgWs200502TrustIssue,
        #[serde(rename = "http://schemas.xmlsoap.org/ws/2005/02/trust/Renew")]
        HttpSchemasXmlsoapOrgWs200502TrustRenew,
        #[serde(rename = "http://schemas.xmlsoap.org/ws/2005/02/trust/Cancel")]
        HttpSchemasXmlsoapOrgWs200502TrustCancel,
        #[serde(rename = "http://schemas.xmlsoap.org/ws/2004/04/security/trust/Issue")]
        HttpSchemasXmlsoapOrgWs200404SecurityTrustIssue,
        #[serde(rename = "http://schemas.xmlsoap.org/ws/2004/04/security/trust/Renew")]
        HttpSchemasXmlsoapOrgWs200404SecurityTrustRenew,
        #[serde(rename = "http://schemas.xmlsoap.org/ws/2004/04/security/trust/Cancel")]
        HttpSchemasXmlsoapOrgWs200404SecurityTrustCancel,
    }
    pub type RequestTypeOpenEnumType = RequestTypeEnumType;
    pub type RequestedAttachedReference = RequestedAttachedReferenceElementType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct RequestedAttachedReferenceElementType {
        #[serde(rename = "SecurityTokenReference")]
        pub security_token_reference: super::wsse::SecurityTokenReferenceType,
    }
    pub type RequestedProofToken = RequestedProofTokenType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct RequestedProofTokenType {
        #[serde(rename(serialize = "wst:BinarySecret", deserialize = "BinarySecret"))]
        pub binary_secret: BinarySecretType,
    }
    pub type RequestedSecurityToken = RequestedSecurityTokenType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct RequestedSecurityTokenType {
        #[serde(
            default,
            rename = "EncryptedData",
            skip_serializing_if = "Option::is_none"
        )]
        pub encrypted_data: Option<EncryptedDataType>,
        #[serde(
            default,
            rename(
                serialize = "wsse:BinarySecurityToken",
                deserialize = "BinarySecurityToken"
            )
        )]
        pub binary_security_token: Option<super::wsse::BinarySecurityTokenType>,
    }
    pub type RequestedSecurityTokenTypeEncryptedData = EncryptedDataType;
    pub type RequestedTokenReference = RequestedTokenReferenceType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct RequestedTokenReferenceType {
        #[serde(rename(serialize = "wsse:KeyIdentifier", deserialize = "KeyIdentifier"))]
        pub key_identifier: super::wsse::KeyIdentifierType,
        #[serde(rename(serialize = "wsse:Reference", deserialize = "Reference"))]
        pub reference: super::wsse::ReferenceType,
    }
    pub type RequestedUnattachedReference = RequestedAttachedReferenceElementType;
    pub type RequestedUnattachedReferenceElementType = RequestedAttachedReferenceElementType;
    pub type TokenType = String;
}

#[allow(dead_code)]
pub mod wsu {
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Serialize, Deserialize)]
    pub struct AttributedDateTimeType {
        #[serde(default, rename = "@Id", skip_serializing_if = "Option::is_none")]
        pub id: Option<String>,
        #[serde(rename = "$text")]
        pub content: String,
    }
    pub type AttributedUriType = AttributedDateTimeType;
    pub type Created = AttributedDateTimeType;
    pub type Expires = AttributedDateTimeType;
    pub type Id = String;
    pub type Timestamp = TimestampType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct TimestampType {
        #[serde(default, rename = "@Id")]
        pub id: Option<String>,
        #[serde(default, rename = "$value")]
        pub content: Vec<TimestampTypeContent>,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub enum TimestampTypeContent {
        #[serde(rename = "Created")]
        Created(AttributedDateTimeType),
        #[serde(rename = "Expires")]
        Expires(AttributedDateTimeType),
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub enum TtimestampFaultType {
        #[serde(rename = "wsu:MessageExpired")]
        WsuMessageExpired,
    }
}

#[allow(dead_code)]
pub mod psf {
    use serde::{Deserialize, Serialize};
    pub type Puid = String;
    pub type PuidType = String;
    pub type Authstate = String;
    pub type AuthstateType = String;
    pub type BrowserCookie = BrowserCookieType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct BrowserCookieCollectionType {
        #[serde(
            default,
            rename = "browserCookie",
            skip_serializing_if = "Vec::is_empty"
        )]
        pub browser_cookie: Vec<BrowserCookieType>,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub struct BrowserCookieType {
        #[serde(default, rename = "@Name")]
        pub name: Option<String>,
        #[serde(default, rename = "@URL")]
        pub url: Option<String>,
        #[serde(rename = "$text")]
        pub content: String,
    }
    pub type BrowserCookies = BrowserCookieCollectionType;
    pub type ConfigVersion = String;
    pub type ConfigVersionType = String;
    pub type Cookies = super::xs::AnyType;
    pub type CredProperties = CredPropertyCollectionType;
    pub type CredProperty = CredPropertyType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct CredPropertyCollectionType {
        #[serde(
            default,
            rename = "credProperty",
            skip_serializing_if = "Vec::is_empty"
        )]
        pub cred_property: Vec<CredPropertyType>,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub struct CredPropertyType {
        #[serde(default, rename = "@Name")]
        pub name: Option<String>,
        #[serde(rename = "$text")]
        pub content: String,
    }
    pub type ExtProperties = ExtPropertyCollectionType;
    pub type ExtProperty = ExtPropertyType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ExtPropertyCollectionType {
        #[serde(default, rename = "extProperty", skip_serializing_if = "Vec::is_empty")]
        pub ext_property: Vec<ExtPropertyType>,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ExtPropertyType {
        #[serde(default, rename = "@IgnoreRememberMe")]
        pub ignore_remember_me: Option<bool>,
        #[serde(default, rename = "@Domains")]
        pub domains: Option<String>,
        #[serde(default, rename = "@Expiry")]
        pub expiry: Option<String>,
        #[serde(default, rename = "@Name")]
        pub name: Option<String>,
        #[serde(rename = "$text")]
        pub content: String,
    }
    pub type Pp = PpHeaderType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct PpHeaderType {
        #[serde(rename = "@xmlns:psf", skip_serializing_if = "Option::is_none")]
        pub xmlns_psf: Option<XmlnsPsfOpenEnumType>,
        #[serde(rename(serialize = "psf:serverVersion", deserialize = "serverVersion"))]
        pub server_version: i32,
        #[serde(rename(serialize = "psf:PUID", deserialize = "PUID"))]
        pub puid: String,
        #[serde(rename(serialize = "psf:configVersion", deserialize = "configVersion"))]
        pub config_version: String,
        #[serde(rename(serialize = "psf:uiVersion", deserialize = "uiVersion"))]
        pub ui_version: String,
        #[serde(rename(serialize = "psf:authstate", deserialize = "authstate"))]
        pub authstate: String,
        #[serde(rename(serialize = "psf:reqstatus", deserialize = "reqstatus"))]
        pub reqstatus: String,
        #[serde(rename(serialize = "psf:serverInfo", deserialize = "serverInfo"))]
        pub server_info: ServerInfoType,
        #[serde(rename(serialize = "psf:cookies", deserialize = "cookies"))]
        pub cookies: super::xs::AnyType,
        #[serde(
            rename(serialize = "psf:browserCookies", deserialize = "browserCookies"),
            skip_serializing_if = "Option::is_none"
        )]
        pub browser_cookies: Option<BrowserCookieCollectionType>,
        #[serde(
            rename(serialize = "psf:credProperties", deserialize = "credProperties"),
            skip_serializing_if = "Option::is_none"
        )]
        pub cred_properties: Option<CredPropertyCollectionType>,
        #[serde(
            rename(serialize = "psf:extProperties", deserialize = "extProperties"),
            skip_serializing_if = "Option::is_none"
        )]
        pub ext_properties: Option<ExtPropertyCollectionType>,
        #[serde(rename(serialize = "psf:response", deserialize = "response"))]
        pub response: super::xs::AnyType,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub enum XmlnsPsfOpenEnumType {
        #[serde(rename = "http://schemas.microsoft.com/Passport/SoapServices/SOAPFault")]
        SoapServicesSoapFault,
    }
    pub type Reqstatus = String;
    pub type ReqstatusType = String;
    pub type Response = super::xs::AnyType;
    pub type ServerInfo = ServerInfoType;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ServerInfoType {
        #[serde(default, rename = "@ServerTime")]
        pub server_time: Option<String>,
        #[serde(default, rename = "@LocVersion")]
        pub loc_version: Option<i32>,
        #[serde(default, rename = "@RollingUpgradeState")]
        pub rolling_upgrade_state: Option<String>,
        #[serde(default, rename = "@Path")]
        pub path: Option<String>,
        #[serde(rename = "$text")]
        pub content: String,
    }
    pub type ServerVersion = i32;
    pub type ServerVersionType = i32;
    pub type UiVersion = String;
    pub type UiVersionType = String;
}
