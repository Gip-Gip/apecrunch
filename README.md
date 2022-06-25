# ApeCrunch, *a calculator for apes...*

## **What is ApeCrunch?**

**ApeCrunch** aims to be a fast, user-friendly Rust/TUI calculator port of the popular [SpeedCrunch](https://speedcrunch.org/) project. While not a copy-paste port, there are plans to implement every function SpeedCrunch supports plus the addition of much more

## **TO-DO:**
 - [ ] Add variables
 - [ ] Add good exponents
 - [ ] Add good roots
 - [ ] Add dedicated square root function
 - [ ] Add parentheses
 - [ ] Add summaries to the top of source files
 - [ ] Properly format comments

## **Changelog:**

### **Version 0.0.1 *in progress***

 - [x] **Added fractional numbers** *(commit eba06ee)*
 - [x] **Added exponents** *(commit c84bc01)*
 - [x] **Added support for negative numbers** *(commit 1ec049a)*
 - [x] **Added ability to scroll through history** *(commit 6aede0c)*
 - [x] **Added ability to select history entries** *(commit fa7ade5)*
 - [x] **Added ability to exit program by pressing Esc** *(commit 6366ec7)*
 - [x] **Added configurable decimal places** *(commit 7b4db5b)*
 - [x] **Added automatic saving of calculations across multiple sessions** *(commit 3abc2cc)*
 - [x] **Config and history files are now stored per OS requirements** *(commit 7ea163c)*
 - [x] **Deuglified layout/tui code** *(commit 6aede0c/6366ec7)*
 - [x] **Enabled crossterm backend** *(commit 6366ec7)*
 - [x] **No longer crashes when empty expression is entered** *(commit 159c78a)*
 - [x] **Made view fullscreen** *(commit 62313aa)*
 - [x] **Made test cases** *(commit ca102c9)*
 - [x] **Ran rustfmt on source code** *(commit 24b37cd)*