# png-to-twmap

Takes a png and builds a teeworlds map out of it.

## Usage

Create a png and call the converter, passing the path as an argument.
It should also work to drag & drop the file onto the exe, if you're on windows.

## Default Colors

| Name       | HTML-Color | preview                             |
| ---------- | ---------- | ----------------------------------- |
| Hookable   | #B97A57    | <div style="color:#B97A57;">▇</div> |
| Unhookable | #FF7F27    | <div style="color:#FF7F27;">▇</div> |
| Air        | #000000    | <div style="color:#000000;">▇</div> |
| Freeze     | #C3C3C3    | <div style="color:#C3C3C3;">▇</div> |
| Spawn      | #3F48CC    | <div style="color:#3F48CC;">▇</div> |
| Start      | #FFC90E    | <div style="color:#FFC90E;">▇</div> |
| Finish     | #22B14C    | <div style="color:#22B14C;">▇</div> |

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
