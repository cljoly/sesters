/*
Sesters: easily convert one currency to another
Copyright (C) 2018-2022  Clément Joly <oss+sesters@131719.xyz>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

pub fn yes_or_no(msg: &str) -> bool {
    loop {
        println!("{msg} [Y/n]");

        let mut line = String::with_capacity(3);
        let _ = std::io::stdin()
            .read_line(&mut line)
            .expect("Couldn’t read input");

        match line.to_ascii_lowercase().as_str() {
            "y\n" | "yes\n" => return true,
            "n\n" | "no\n" => return false,
            _ => println!("Please enter “yes” or “no”"),
        }
    }
}
