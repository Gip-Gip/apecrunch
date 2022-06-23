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
 - [ ] Add the ability to adjust the number of decimal places displayed
 - [ ] Allow the user to quit the program without hitting ctrl+c
 - [ ] Make the theme less ugly
 - [ ] Make test cases
 - [ ] Properly format comments
 - [ ] Store theme in ~/.apecrunch/theme.toml
 - [ ] Store history files in ~/.apecrunch/history/
 - [ ] Store the parsed tokens of calculations in history instead of the printed result

## **Changelog:**

### **Version 0.0.1 *in progress***

 - [x] **Added fractional numbers** *(commit eba06ee)*
 - [x] **Added exponents** *(commit c84bc01)*
 - [x] **Added support for negative numbers** *(commit 1ec049a)*
 - [x] **Added ability to scroll through history** *(commit 6aede0c)*
 - [x] **Added ability to select history entries** *(commit fa7ade5)*
 - [x] **Added automatic saving of calculations across multiple sessions** *(commit 3abc2cc)*
 - [x] **Deuglified layout code** *(commit 6aede0c)*
 - [x] **No longer crashes when empty expression is entered** *(commit 159c78a)*
 - [x] **Made view fullscreen** *(commit 62313aa)*
 - [x] **Ran rustfmt on source code** *(commit 24b37cd)*