enum Stat {
    Ok,
    Fail
}

struct PhotoRaw {
    title: String,
    ispublic: u32,
    url_k: String, // TODO: Add serde Url crate
    height_k: u32,
    width_k: String
}

struct Photo {
    title: String,
    ispublic: bool,
    url_k: String, // TODO: Add serde Url crate
    height_k: u32,
    width_k: u32
}

struct PhotosResponse {
    photo: Vec<PhotoRaw>,
    stat: Stat
}
