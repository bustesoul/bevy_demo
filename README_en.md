# Text-based RPG Framework

This is a text/console RPG game framework built with Rust and the Bevy game engine. The project features a modular design following Bevy's Entity-Component-System (ECS) pattern, implementing core RPG features like character stats, equipment, and item usage.

## Core Features

-   **Character System**: Complete character attributes, including HP, attack, defense, level, and experience. Features an automatic leveling system.
-   **Equipment System**: Four equipment slots (head, body, weapon, accessory) that dynamically affect character stats.
-   **Inventory System**: Manages player items with support for stacking. Seamlessly integrated with the equipment system.
-   **Data-Driven**: Game data, such as items, is loaded from external RON (`.ron`) files.
-   **Event-Driven Architecture**: Modules are decoupled through an event-based system. For example, changing equipment automatically triggers a recalculation of character stats.
-   **Command-Line Interface**: All game interactions are handled through a simple text-based command interface.

## Project Structure

The project is organized into several core modules:

-   `main.rs`: The application entry point, which assembles all plugins.
-   `core`: Defines global states, events, and resources.
-   `character`: Manages character stats, leveling, and related logic.
-   `data`: Handles loading game data from RON files.
-   `inventory`: Implements the player's backpack and item management.
-   `equipment`: Manages the character's equipment slots.
-   `interface`: Provides the command-line interface for user interaction.

## Detailed Documentation

- [Project Architecture (Chinese)](doc/arch_cn.md)
- [Project Features (Chinese)](doc/cur_state.md)

## How to Run

1.  Clone the repository:
    ```bash
    git clone https://github.com/bustesoul/bevy_demo.git
    cd bevy_demo
    ```

2.  Run the application:
    ```bash
    cargo run
    ```

## Available Commands

Once the game is running, you can interact with it using the following commands:

-   `help` - Show available commands.
-   `status` - View current player status.
-   `stats` - Display detailed character attributes.
-   `inventory` - Show items in your backpack.
-   `items` - List all defined items in the game.
-   `give <item_id> <count>` - Add an item to your inventory (for debugging).
-   `equip <slot> <index>` - Equip an item from your inventory.
-   `unequip <slot>` - Unequip an item.
-   `use <index>` - Use an item from your inventory.
-   `exit` - Quit the game.
