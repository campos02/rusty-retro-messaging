use crate::errors::contact_verification_error::ContactVerificationError;
use crate::models::transient::authenticated_user::AuthenticatedUser;

pub fn verify_contact(
    authenticated_user: &AuthenticatedUser,
    email: &String,
) -> Result<(), ContactVerificationError> {
    if let Some(contact) = authenticated_user.contacts.get(email) {
        if *authenticated_user.blp == "BL" && !contact.in_allow_list {
            return Err(ContactVerificationError::ContactNotInAllowList);
        }

        if contact.in_block_list {
            return Err(ContactVerificationError::ContactInBlockList);
        }

        if let Some(presence) = &authenticated_user.presence {
            if **presence == "HDN" {
                return Err(ContactVerificationError::UserAppearingOffline);
            }
        } else {
            return Err(ContactVerificationError::UserAppearingOffline);
        }
    } else if *authenticated_user.blp == "BL" && *email != *authenticated_user.email {
        return Err(ContactVerificationError::ContactNotInAllowList);
    }

    Ok(())
}
