# ApeCrunch, *a calculator for apes...*

## **What is ApeCrunch?**

**ApeCrunch** aims to be a fast, user-friendly Rust/TUI calculator port of the popular [SpeedCrunch](https://speedcrunch.org/) project. While not a copy-paste port, there are plans to implement every function SpeedCrunch supports plus the addition of much more

## **TO-DO:**

 - [ ] Add dedicated square root function
 - [ ] Add ability to retrieve the answers of previous expressions
 - [ ] Add built in functions like sin, cos, tan, etc.
 - [ ] Make README pretty

## **Changelog:**

### **Version 0.0.3**
 - [X] **Fixed bug where inserting a previous history entry would cause a panic** *(commit 5a0152e)*

### **Version 0.0.2**
- [X] **Added good roots** *(commits 204f105/032b169/c98ee95/0298679/c980033)*
- [X] **Added variables** *(commits 2bf6780/337064e)*
    - [X] **Added automatic storing of variables to history files** *(commit 52e7c70)*
    - [X] **Added variable name checking** *(commit 4170b3e)*
- [X] **Added history file version check** *(commit 8ab539d)*
- [X] **Improved the code that postfixes three dots to numbers if there is a loss of precision converting the number to a string** *(commit c9e1613)*
- [X] **Fixed crash when reading corrupt/incompatible history files** *(commit 8ab539d)*
- [X] **Fixed order of operation** *(commit 2e31175)*

### **Version 0.0.1**

 - [x] **Added fractional numbers** *(commit eba06ee)*
 - [x] **Added exponents** *(commit c84bc01)*
 - [x] **Added support for negative numbers** *(commit 1ec049a)*
 - [x] **Added ability to scroll through history** *(commit 6aede0c)*
 - [x] **Added ability to select history entries** *(commit fa7ade5)*
 - [x] **Added ability to exit program by pressing Esc** *(commit 6366ec7)*
 - [x] **Added configurable decimal places** *(commit 7b4db5b)*
 - [x] **Added automatic saving of calculations across multiple sessions** *(commit 3abc2cc)*
 - [x] **Added parenthesis** *(commit a33a59c)*
 - [x] **Added command line arguments** *(commit 7f31193)*
 - [x] **Added properly formatted comments for rustdoc** *(commit 031bdb4)*
 - [x] **Changed to bincode for history storage** *(commit 7f31193)*
 - [x] **Config and history files are now stored per OS requirements** *(commit 7ea163c)*
 - [x] **Deuglified layout/tui code** *(commit 6aede0c/6366ec7)*
 - [x] **Enabled crossterm backend** *(commit 6366ec7)*
 - [x] **No longer crashes when empty expression is entered** *(commit 159c78a)*
 - [x] **Made view fullscreen** *(commit 62313aa)*
 - [x] **Made test cases** *(commit ca102c9)*
 - [x] **Made code even more idiomatic** *(commit d735543)*
 - [x] **Sessions are now sorted by start date upon loading** *(commit 6911f74)*
 - [x] **Ran rustfmt on source code** *(commit 24b37cd)*