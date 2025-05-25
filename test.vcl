vcl 4.1;
import std;

backend test {
        .host = "testbackend:8080";
		.between_bytes_timeout = 2s;
		.first_byte_timeout = 2s;
		.connect_timeout = 1s;
}
