This is a place for notes for the engine.

I have decided to name it leaf or leaf Engine. I just like plants and leaf is short.

## 4/12/21

So I converted the project to a library project, created some imports as of now,
and moved startup and shutdown into the application. I think I'm starting to run into
"fighting the borrow checker" as they say. I'm having a hard time accessing and iterating
over the managers vector inside the application because of mutability issues. I think ideally
the managers would be stored using ECS so that each manager is essentially just an index that maps
to a set of functionality. I'd really have to think that one through though.

It would probably good to implement logging soon so that I can move away from print
statements into a more structured logging manager. There's a part of me that wants to
do that as a separate library, but I know that's going to be complex, so I think I'll hold off until
I need it. I'm imagining a logging manager that can write to the terminal in various colors
and also possibly files. I think the immediate next step is probably to get the window loop happening
in the render manager or however makes sense so that the application can actually run.

Also I just reordered this so new things are at the top. Also this is going to
potentially throw off my line count in the github contributions early on while
the project is small, but who cares.

## 4/11/21

Created a lib.rs. I want to actually delete the main.rs function and figure out
how to convert this from a binary project to a library project. It hit me today
that I am trying to create both an engine and a game, so I need to actually keep
them separate from the get go. I think my path forwards is to figure out how to
flesh out the create_application function in the application manager, move all of
the sub manager initialization to there, and figure out what the deal is with lib.rs
files. Then, I can create an empty shell of a game using leaf that I'll use to test
everything.

In a nutshell, next steps are:
1. move submanager initialization to create_application function.
2. research lib.rs file
3. create game project with same end result that uses leaf to get an application
and run

## 4/14/21

Currently working on creating a window class. I don't really like the way the
managers and render code is stored right now. The window class should ideally abstract
and detect which manager is running, but also honestly glutin might already handle that.
in any case, I'm having trouble getting a window struct to store an event loop
and other glutin types and i'm not really sure why.

## 4/19/21

Added the beginnings of an observer manager in rust. Basically it's going to be an observer
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
the render manager. I also might actually start using Github or check out Trello because
there's a couple more things that need fixing.

## 4/28/21

So the context and event loop and observer pattern has been giving me significant trouble.
I see that is what I've been working on for almost two weeks now based on this file. Basically,
I wanted the main application to hold a window class that held a context, and the context would
hold the event loop, which would then notify the main application via weak reference on each update
so that  the application could then update each of the submanagers. I have come to learn that involves
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
render manager because as of now, it won't do anything. For now i'll keep the code because I
simply cannot imagine a game engine without a render manager, so i'm sure i'll need it eventually.
also it's possible I refactor later into the render manager anyways.

well I guess I'll just go fuck myself because I just fixed the whole context event loop consumption thing
right after I typed that. Literally clicked out of this file to the window file and fixed it.
just used mem::replace() instead. however, the observer manager is still broken, so there's that.
Yeah pretty sure I should still just do away with the observer pattern. The other issue with
it is really the only observer of that event is the application. so it's a one to one relationship anyways.

just committed and pushed and gonna fuck everything up and see if i can get it working.
Yeah that worked.

## 5/4/21

Started a math library that's basic, but I got sidetracked learning rust's module manager.
In any case, abacus now exists. I wanted to just call it math, but there's already a math crate.
There's probably already an abacus crate, but fuck it. Next step is to draw a triangle but using the
machinery of the engine. I'm thinking it might be about time to flesh out the render manager.

## 5/7/21

Moved the math library under the engine because apparently glium can't implement vertex types
onto structures not defined in this crate, but that's fine. that's how i was originally going
to do it anyways. I'm implementing geometries and renderables right now, and i originally
opened this to say I should probably implement a geometry trait even if it just implements
a create function so that I can use it as a type, but i think i'll just do that now.
actually no i'm not going to do that until i need it. i don't want to implement anything
without good reason and "because i might need it" seems superfluous. a renderable, on
the other hand, definitely seems like something i want a trait for. This way I can accept
vectors of renderables and know they all have initialize and draw calls etc.
also i should probably start writing comments. I comment code way more at work
apparently. I've now written out the renderable initialization with buffers and whatever.
the next task is to figure out how to structure the draw call.

## 5/11/21

Ok so i have created the renderable and it works. there's a triangle on the screen. great.
now I have to actually think this thing through. I've recently been reading the mythical
man month and specifically about the second-manager effect.  Basically, the second manager
an engineer builds tends to be bloated and over engineered because of overconfidence.
I'm especially interested in keeping this as lightweight as possible because I want it
to be efficient. That being said, I'm genuinely unsure I even have enough experience
to know whether or not i'm over engineering a solution. Currently, there's a core application,
and it spins up a physics manager and a render manager. Neither of those managers currently
do a single thing. Currently, the main context and loop exists inside the main application class.
I believe the reason for that is elsewhere in these notes. The rendering module holds geometries
and renderables, and the renderables can be created and rendered. The question sort of naturally becomes
how do I render multiple objects? How should I store them? How should I render them? What is the
purpose of a render manager in the first place? in trying to answer these, am I going to over engineer this?
I don't think so. I don't think I am smart enough to over engineer anything.
