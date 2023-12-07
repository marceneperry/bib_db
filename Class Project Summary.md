What I learned from building this project.

- Interesting:
    - Building something from scratch that I have used in the real world was fun, but challenging at the same time.
    - I have a better understanding of the complexity involved with writing a `RDBMS`. There is way more detail and work
      that would go into building this out
      even more.
    - `ratatui`! This crate was fun for a personal project. For a more professional project, I would probably use wasm

- Difficult:
    - I started out using `async`, bBut I had a lot of problems getting the program up and running. So I switched to
      synchronous
      because
      I wasn't too concerned about multiple access to the database.
    - Making a fully fledged `RDBMS` would require using async. In the next version I would use async.
    - Sometimes adding a new function was challenging. But it was also helpful in helping me understand program design.

- Easy:
    - I think I was able to utilize Structs and Enums fairly effectively.
    - UI was fun to play around with. Although I didn't spend a lot of time changing the colors, I was able to set up
      the different sections of the TUI the way that I wanted them to look.

- Other:
    - Finishing up the project I ran `clippy` and `clippy::pedantic`. I made all the `clippy` changes. But when it came
      to some of the pedantic recommendations I will wait until another refactor to make those changes. Changing some of
      the `pass by reference` to `pass by value` could result in some breaking changes.
  