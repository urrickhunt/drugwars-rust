### 40th Anniversary Drugwars in Rust 🦀
![drugwars](https://github.com/user-attachments/assets/9c7e1165-9b67-4ee8-8068-af12a9f34518)

Drugwars, the classic text-based game, is back & rewritten in Rust for its 40th Anniversary. 

This cross-platform version stays true to the original experience, featuring familiar gameplay & updated terminal compatibility.

Single Key Commands like the original ensure smooth gameplay with quick input handling.

### Installation

`cargo install drugwars-rust`

### Building

`git clone https://github.com/urrickhunt/drugwars-rust`

- normal release

`cargo build --release`

- lto release

`cargo build --profile release-lto`

- normal install

`cargo install --path .`

- lto install

`cargo install --path . --profile release-lto`

- run

`drugwars-rust`

- run on git bash mintty

`winpty drugwars-rust`

![GM](https://github.com/user-attachments/assets/afbbe054-b0ed-4214-8644-2d962995639b)

