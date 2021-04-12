This is a place for notes for the engine.

I have decided to name it Luto or Luto Engine. Lutum is latin for soil, and
luto is the ablative case which means from the soil. I just like plants.


4/11/21

Created a lib.rs. I want to actually delete the main.rs function and figure out
how to convert this from a binary project to a library project. It hit me today
that I am trying to create both an engine and a game, so I need to actually keep
them separate from the get go. I think my path forwards is to figure out how to
flesh out the create_application function in the application manager, move all of
the sub system initialization to there, and figure out what the deal is with lib.rs
files. Then, I can create an empty shell of a game using luto that I'll use to test
everything.

In a nutshell, next steps are:
1. move subsystem initialization to create_application function.
2. research lib.rs file
3. create game project with same end result that uses luto to get an application
and run
