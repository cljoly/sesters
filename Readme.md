
<div id="home-anchor"></div>
<div align="center">

  <img
       src="https://raw.githubusercontent.com/cljoly/sesters/master/logo76.png"
       height="70"
       width="70"
       >


<h1 alig="center">
  Sesters
</h1>

💱 Fast, offline currency converter 💴 💷 💶 💵
</div>

<p align="center">
<a href="https://github.com/cljoly/sesters#getting-started"><img src="https://img.shields.io/badge/🚀 getting-started-yellow?style=flat-square" alt="Crates.io" /></a> </a><a href="#contribute"><img src="https://img.shields.io/github/license/cljoly/sesters.svg?color=blueviolet&label=contribute&style=flat-square" alt="LICENCE" /></a> <a href="https://crates.io/crates/sesters"><img src="https://img.shields.io/crates/v/sesters.svg?color=blue&style=flat-square" alt="Crates.io" /></a> <a href="https://crates.io/crates/sesters"><img alt="undefined" src="https://img.shields.io/crates/d/sesters.svg?color=brightgreen&style=flat-square"></a>
</p>

******************************************

## Getting started

Install the latest version:

```
$ cargo install sesters
```

Exemple of plain text conversion:
```
$ sesters convert a price burried 1 USD in text
USD 1.00 ➜ EUR 0.89
$ sesters convert -- -1 €
EUR -1.00 ➜ USD -1.10
$ sesters convert
I can type my price and press enter EUR lorem ipsum 2356
EUR 2345.00 ➜ USD 2586.53
```

## Features

🏗️ This is a work in progress, only checked features are implemented yet.

- [X] Find prices in plain text with several currencies
- [X] Store exchange rates locally
- [X] Retrieve exchange rate (partial)
  - [X] Cache retrieved rate
  - [ ] More sources to be added ![GitHub issues by-label](https://img.shields.io/github/issues/cljoly/sesters/rate-source.svg)
- [ ] Save recent searches
  - [ ] Display this history in a table

### Maybe

- [ ] GUI with [azul.rs](https://azul.rs/)

## Contribute

Contributions are welcome!

<p align="center">
<a href="https://github.com/cljoly/sesters/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22"><img src="https://img.shields.io/github/issues/cljoly/sesters/good%20first%20issue" alt="Good first issues" /></a>
<a href="https://github.com/cljoly/sesters/actions/"><img src="https://github.com/cljoly/sesters/workflows/Continuous%20Integration/badge.svg" alt="Continuous Integration" /></a>
</p>


## About the name

Inspired by this [coin](https://en.wikipedia.org/wiki/Sestertius).

<p><a href="https://commons.wikimedia.org/wiki/File:Sestertius_Hostilian-s2771.jpg#/media/File:Sestertius_Hostilian-s2771.jpg"><img src="https://upload.wikimedia.org/wikipedia/commons/f/f3/Sestertius_Hostilian-s2771.jpg" alt="Sestertius Hostilian-s2771.jpg"></a><br>By Classical Numismatic Group, Inc. <a rel="nofollow" class="external free" href="http://www.cngcoins.com">http://www.cngcoins.com</a>, <a href="http://creativecommons.org/licenses/by-sa/3.0/" title="Creative Commons Attribution-Share Alike 3.0">CC BY-SA 3.0</a>, <a href="https://commons.wikimedia.org/w/index.php?curid=380116">Link</a></p>

## Licence

> Copyright (C) 2018-2019  Clément Joly <oss+sesters@131719.xyz>
> 
> This program is free software: you can redistribute it and/or modify
> it under the terms of the GNU General Public License as published by
> the Free Software Foundation, either version 3 of the License, or
> (at your option) any later version.
> 
> This program is distributed in the hope that it will be useful,
> but WITHOUT ANY WARRANTY; without even the implied warranty of
> MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
> GNU General Public License for more details.
> 
> You should have received a copy of the GNU General Public License
> along with this program.  If not, see <https://www.gnu.org/licenses/>.
,
