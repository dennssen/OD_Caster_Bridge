# OD Caster Bridge
**The official casting overlay solution for Orion Drift esports**

OD Caster Bridge is the only application that enables casting Orion Drift with professional overlays, providing casters with real-time game data integration and an intuitive control interface.

## Overview
OD Caster Bridge bridges the gap between Orion Drift's game API and your broadcast overlays, making it easy and performant to deliver high-quality esports coverage. The app handles all the heavy lifting of data management while giving you full control over team information, scores, and match details through a user-friendly GUI.

## Features
- **Real-time Game Data**: Seamlessly connects to Orion Drift's API for live match data
- **Team Management**: Edit team names and logos on the fly
- **Score Override**: Manually correct round scores when the API reports incorrect data
- **Intuitive GUI**: Simple, straightforward interface designed for live production environments
- **Performance Optimized**: Built for smooth, reliable operation during critical broadcast moments
- **Template Overlay Included**: Get started quickly with this [template overlay](https://github.com/dennssen/OD-Overlay-Template) or use it as a learning resource for building your own

## Requirements
- Orion Drift spectator with the [**Caster Camera**](https://github.com/dennssen/CasterCamera) script [(v2.3.0 or above)](https://github.com/dennssen/CasterCamera/releases/latest)
- OBS Studio or similar broadcasting software

> **Important**: OD Caster Bridge is only compatible with the Caster Camera spectator script. Ensure you have this script enabled while using the app.

## Getting Started
1. Launch the Orion Drift spectator and enable the Caster Camera spectator script
2. Find the arena you want to cast and make sure "Send Overlay Data" is enabled in the Caster Camera script
3. Start OD Caster Bridge
4. Setup your OBS scenes
5. Add your overlays as a browser source in OBS using local files
6. Configure team names and upload logos through the OD Caster Bridge UI
7. Start casting!

If you're new to creating overlays, check out the [template overlay](https://github.com/dennssen/OD-Overlay-Template) to get started or learn how the system works.

## Migrating from the old method
If you were previously using the old method of setting up overlays and want to switch to this new version follow these steps:
1. Update the Caster Camera to version [v2.3.0 or above](https://github.com/dennssen/CasterCamera/releases/latest)
2. Delete the `dennssen.overlayInfo.luau` script
3. Delete the old `Overlay` folder gotten from the `OverlayWebsite` repo (Save your custom-made overlays if you have any)
4. Install the new [template overlay](https://github.com/dennssen/OD-Overlay-Template) (Or update your custom-made overlays to work with the new api)
5. Swap your browser sources in OBS with the new overlays
6. Done!

## Custom Overlays
Previously made custom overlays are not compatible with the OD Caster Bridge straight out of the box. However, it should be a relatively simple change.
A wiki page is being made on this repository to document the new api and how to use it. 
Until then, I recommend looking at the [template overlay](https://github.com/dennssen/OD-Overlay-Template) to see the new approach.

## Roadmap
I'm actively developing OD Caster Bridge with several exciting features planned:

- [ ] **Overlay Hosting & Switching**: Host overlays directly through the app's HTTP server and switch between different overlays without manually changing URIs in OBS
- [ ] **Manual Round Addition**: Add rounds manually in addition to the current delete and edit functionality
- [ ] **Configurable Keybinds**: Quick actions like increasing/decreasing scores on previous rounds, and more
- [ ] **OBS WebSocket Integration**: Automatic scene switching and source control triggered by in-game events like goals scored and round transitions

## Contributing
Contributions are welcome! If you'd like to contribute to OD Caster Bridge, please follow the [Rust Style Guide](https://doc.rust-lang.org/style-guide/) when submitting pull requests.

## Support
For issues, feature requests, or questions, please open an issue on this repository.

## License
MIT License

Copyright (c) 2026 dennssen

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.