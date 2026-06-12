use runique::prelude::*;

extend! {
    table: "eihwaz_users",
    fields: {
        bio: textarea,
        avatar: image [upload_to: "avatars/"],
        website: url,
        phone: phone,
        birth_date: date,
        is_verified: bool [default: false],
        linkedin: url,
        job_title: text,
    }
}
