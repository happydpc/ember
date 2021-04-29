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

## 4/28/21

So the context and event loop and observer pattern has been giving me significant trouble.
I see that is what I've been working on for almost two weeks now based on this file. Basically,
I wanted the main application to hold a window class that held a context, and the context would
hold the event loop, which would then notify the main application via weak reference on each update
so that  the application could then update each of the subsystems. I have come to learn that involves
cyclical references, albeit some of those might be reference counted or have other interior mutablity
traits. This is because the application holds the window, which holds the context, which holds the application.
that is apparently not recommended in rust. Beyond that, the other issue is that the event loop run function consumes
the loop and its containing context in a closure. I cannot transfer ownership of the event loop from
the window to the context because the window is at most a mutable reference. since it is owned by the
main application class it does not own itself, so I cannot transfer ownership of its fields. However,
there is the workaround of the standard library take function, which would work (i think), but
the event loop does not implement the default trait. I'm realizing now I might be able to
just implement the default trait for the event loop function, but the simplest approach
seems to just be to move some or all of the context into the main application. If I move the
context to the application and store the event loop on the application, I should be able to move it
to the context in a run function and then from there just call a self.update() or something
similar on the application. This does, however, make me wonder about the functionality of the
render system because as of now, it won't do anything. For now i'll keep the code because I
simply cannot imagine a game engine without a render system, so i'm sure i'll need it eventually.
also it's possible I refactor later into the render system anyways.

well I guess I'll just go fuck myself because I just fixed the whole context event loop consumption thing
right after I typed that. Literally clicked out of this file to the window file and fixed it.
just used mem::replace() instead. however, the observer system is still broken, so there's that.
Yeah pretty sure I should still just do away with the observer pattern. The other issue with
it is really the only observer of that event is the application. so it's a one to one relationship anyways.

just committed and pushed and gonna fuck everything up and see if i can get it working.
Yeah that worked.
