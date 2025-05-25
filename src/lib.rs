use zed_extension_api::{self as zed, register_extension};

struct VarnishExtension;

impl zed::Extension for VarnishExtension {
    fn new() -> Self {
        Self
    }
}

register_extension!(VarnishExtension);
