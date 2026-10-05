use runique::prelude::*;

extend! {
    table: "eihwaz_users",
    fields: {
        bio: textarea [nullable],
        avatar: image [nullable, upload_to: "avatars/"],
        website: url [nullable],
        phone: phone [nullable],
        birth_date: date [nullable],
        is_verified: bool [nullable, default: false],
    }
}
