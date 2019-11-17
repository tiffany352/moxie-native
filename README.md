# Moxie Native

**Warning: This is a work in progress.**

This is a framework for building GUI applications written in Rust. It
renders natively using Webrender, instead of relying on other UI
toolkits like Gtk or on a web browser. In that sense, it is kind of like
React Native, but targeting Rust instead of Javascript.

The front-facing API is built using Moxie, a framework for declaratively
defining UI similar to React.

## Features

- Relatively small footprint (~11MB for a hello world app).
- Declaratively specify UI instead of manually managing state.
- Not based on immediate mode UI.
- Targetted towards real world desktop applications.

## Future plans

- Styling (similar to CSS, but specified using Rust macros).
- More powerful layouts (flexbox, flexgrid, etc).
- More user input, such as text fields.
- Support for more OSes than just Windows.

## License

Mozilla Public License, version 2.

## Contributing

- Agree to license your code under the terms of the license.
- Follow the code of conduct.
- Check the issues and projects tabs for things, or ask me (@tiffany352).
