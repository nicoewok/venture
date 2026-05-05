# Venture

Venture is a terminal-based productivity tool that gamifies your work sessions using Medieval RPG tropes. Instead of a standard countdown timer, your work hours are represented as a physical march across a winding ASCII trail toward a distant castle. Your tasks are "monsters" you must slay to complete your quest.


## Getting Started

### Prerequisites
*   **Rust:** You must have the [Rust toolchain](https://www.rust-lang.org/tools/install) installed.
*   **dotdo (Optional):** If you use [dotdo](https://github.com/nicolang0416/dotdo), another TUI app of mine, Venture will automatically pull your active tasks into the game.

## Install (Linux)

Download the deb file from the [releases](https://github.com/nicoewok/venture/releases) page and install it using `sudo apt install venture_*.deb`.

## The Gameplay Loop
Venture is designed to stay open in a small terminal pane or side monitor while you work. The loop consists of four distinct phases:

### 1. The Setup
When you launch the app, you start at the Setup screen.

1. Set Goal: Enter the number of hours you intend to work.
2. Set Monsters: Enter the number of monsters (*tasks*) you intend to fight (*complete*).

### 2. The Tavern

In the tavern you pick (and/or create) the monsters you want to slay during this quest.
Once you have selected your monsters, you can start the quest by pressing `[Enter]`.

### 2. The March (Work mode)
Once the quest begins, your character appears on a winding trail.

- Progress: As time passes, the world scrolls upward, you walk further and further towards the castle.
- Visuals: ASCII trees, mountains, and rivers are generated as you "walk."
- The Castle: A castle sits at the top of the trail, representing your daily goal.

### 3. The Battle
When you are ready to complete a task, you can select a monster to fight. Press S to view your not-yet-slain monsters.

**Combat:** Selecting a monster enters a battle scene. Press `[S]` again to slay the monster or `[F]` to flee back to the trail.

### 4. The Castle
Once your elapsed time meets your goal, you reach the castle.

The trail stops moving, but you can remain at the gates to finish off any remaining monsters.

**Finish:** Press C to explicitly complete the quest and exit.

## Controls

* Arrow Keys: Navigate menus and lists.
* S: Open Slay menu / Slay monster.
* F: Flee from battle.
* P: Pause the march.
* C: Complete quest (only available after beating all monsters or completing the march).
* Q: Quit the application.


### Dotdo integration

Dotdo is a simple cli todo-list tool I created.
You can find it here: [https://github.com/nicolang0416/dotdo](https://github.com/nicolang0416/dotdo)

If you have dotdo installed, Venture will automatically get your active tasks and let's you select them as monsters when you start a session.


### Build and Run
If you want to build the project yourself:

1. Clone the repository.
2. Build and run the project:
```bash
cargo run
```


### ASCII Art

Alligator by [Joan G. Stark (Spunk)](https://www.asciiart.eu/art/ce227a0434b9e8ed)

Wolf by [Unknown](https://www.asciiart.eu/art/80c90fbd73cd8011)

Dragon by [Unknown](https://www.asciiart.eu/art/30abc3bbb104d184)