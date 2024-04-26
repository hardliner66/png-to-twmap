# png-to-twmap

Takes a png and builds a teeworlds map out of it.

## Usage

Create a png and call the converter, passing the path as an argument.
It should also work to drag & drop the file onto the exe, if you're on windows.

## Default Colors

| Name       | HTML-Color | preview                                                                     |
| ---------- | ---------- | --------------------------------------------------------------------------- |
| Hookable   | #B97A57    | <img src="./assets/hookable.png" alt="brown" width = 16px height = 16px>    |
| Unhookable | #FF7F27    | <img src="./assets/unhookable.png" alt="orange" width = 16px height = 16px> |
| Air        | #000000    | <img src="./assets/air.png" alt="black" width = 16px height = 16px>         |
| Freeze     | #C3C3C3    | <img src="./assets/freeze.png" alt="light grey" width = 16px height = 16px> |
| Spawn      | #3F48CC    | <img src="./assets/spawn.png" alt="indigo" width = 16px height = 16px>      |
| Start      | #FFC90E    | <img src="./assets/start.png" alt="gold/yellow" width = 16px height = 16px> |
| Finish     | #22B14C    | <img src="./assets/finish.png" alt="green" width = 16px height = 16px>      |

## Configuration

To configure which color gets mapped to which entity id, first export the default config:

```sh
png-to-twmap --print-default-mappings > config.rsn
```

Then you edit the config file and add a mapping from an RGBA color to an entity id.

There are a few pre-defined entity types that are named:
- Empty
- Unookable
- Hookable
- Freeze
- Spawn
- Start
- Finish

If you need some other entity, you can use `Custom` with the appropriate id.
For instance:
```rs
Custom(123)
```
