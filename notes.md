This is a place for notes for the engine.

I have decided to name it leaf or leaf Engine. I just like plants and leaf is short.

## 4/12/21

So I converted the project to a library project, created some imports as of now,
and moved startup and shutdown into the application. I think I'm starting to run into
"fighting the borrow checker" as they say. I'm having a hard time accessing and iterating
over the systems vector inside the application because of mutability issues. I think ideally
the systems would be stored using ECS so that each system is essentially just an index that maps
to a set of functionality. I'd really have to think that one through though.

It would probably good to implement logging soon so that I can move away from print
statements into a more structured logging system. There's a part of me that wants to
do that as a separate library, but I know that's going to be complex, so I think I'll hold off until
I need it. I'm imagining a logging system that can write to the terminal in various colors
and also possibly files. I think the immediate next step is probably to get the window loop happening
in the render system or however makes sense so that the application can actually run.

Also I just reordered this so new things are at the top. Also this is going to
potentially throw off my line count in the github contributions early on while
the project is small, but who cares.

## 4/11/21

Created a lib.rs. I want to actually delete the main.rs function and figure out
how to convert this from a binary project to a library project. It hit me today
that I am trying to create both an engine and a game, so I need to actually keep
them separate from the get go. I think my path forwards is to figure out how to
flesh out the create_application function in the application manager, move all of
the sub system initialization to there, and figure out what the deal is with lib.rs
files. Then, I can create an empty shell of a game using leaf that I'll use to test
everything.

In a nutshell, next steps are:
1. move subsystem initialization to create_application function.
2. research lib.rs file
3. create game project with same end result that uses leaf to get an application
and run

## 4/14/21

Currently working on creating a window class. I don't really like the way the
systems and render code is stored right now. The window class should ideally abstract
and detect which system is running, but also honestly glutin might already handle that.
in any case, I'm having trouble getting a window struct to store an event loop
and other glutin types and i'm not really sure why.

## 4/19/21

Added the beginnings of an observer system in rust. Basically it's going to be an observer
trait and an ObserverQueue struct that acts as the subject. The class that would be a subject Will
just hold an ObserverQueue, and observers will register themselves in a class' ObserverQueue.
I don't really like this implementation, but i'm trying to get something up and working so that I can have
the context class send update triggers to the main application from the context object.
I'm not sure if that's how it's supposed to work or what. It feels weird to me the driving class
is ultimately going to be the window context event loop. It feels like you should be able to
have the main application tell the window to poll for events and update. It is very possible that
I rewrite that structure later. It's very possible I rewrite huge portions of this later I guess.
Currently I'm just putting things together to learn. There is a section in the game engine book on
this so i'll read that and see how it goes.

## 4/22/21

Finished a first pass at the observer pattern. Currently the window64 class is a subject
and the application manager is an observer. the window class holds an Rc with a RefCell
of a dyn observer so it can mutate it. I think the context needs to be the subject. Pretty sure
i can just have the application register itself as an observer to the context by accessing it on
the render system. I also might actually start using Github or check out Trello because
there's a couple more things that need fixing.
