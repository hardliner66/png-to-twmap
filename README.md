# png-to-twmap

Takes a png and builds a teeworlds map out of it.

## Usage

## Configuration

To configure which color gets mapped to which entity id, first export the default config:

```sh
png-to-twmap export-mappings > config.rsn
```

Then you edit the config file and add a mapping from an RGBA color to an entity id.

There are a few pre-defined entity types that are named:
- Empty
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
