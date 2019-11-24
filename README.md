# Moxie Native

**Warning: This is a work in progress and not yet usable for nontrivial applications.**

This is a framework for building GUI applications written in Rust. It
renders using Webrender, instead of relying on other UI toolkits like
Gtk or on a web browser. This gives you control over how your
application looks, but may lose some native look and feel.

The front-facing API is built using Moxie, a framework for declaratively
defining UI similar to React.

## Features

- Relatively small footprint (~11MB for a hello world app).
- Declaratively specify UI instead of manually managing state.
- Not based on immediate mode UI.
- Targetted towards real world desktop applications.
- Styling system for specifying the appearance of elements.

## Future plans

- More powerful layouts (flexbox, flexgrid, etc).
- More user input, such as text fields.
- See here for the current roadmap: https://github.com/tiffany352/moxie-native/projects/1

## License

Mozilla Public License, version 2.

## Contributing

- Agree to license your contribution under the terms of the [license](./LICENSE-MPL).
- Follow the [code of conduct](./CODE_OF_CONDUCT.md).
- Check the issues and projects tabs for things to work on, or ask me (@tiffany352).
