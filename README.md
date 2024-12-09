# Harmonia

A sleek lyrics visualizer built using **Tauri**, **React**, and **TypeScript**, designed for Linux.

## Screenshots

![image](https://github.com/user-attachments/assets/7bda8cd9-5f98-4c9f-b334-6da4dcae1826)
![image](https://github.com/user-attachments/assets/304472b7-ad3b-4bb3-b6c8-ef5998ba8ac4)


## Getting Started

### Prerequisites

This project uses [playerctl](https://github.com/altdesktop/playerctl) under the hood to query the current players. Make sure you have it installed.

### Compile from source

> [!NOTE]
> There is already a compiled release.

1. Clone the repository:

   ```bash
   git clone https://github.com/shenawy29/harmonia.git
   cd harmonia
   ```

2. Install dependencies:

   ```bash
   npm install
   ```

3. Build:

   ```bash
   npm run tauri build
   ```

4. Run:

   ```bash
   ./src-tauri/target/release/harmonia
   ```


## Note for Hyprland users

If you want the exact look I have in the screenshots, add the following to your Hyprland config file:

```conf
"noblur, class: harmonia"
"noshadow, class: harmonia"
"noborder, class: harmonia"
```

## Contribution

Contribution is, as always, welcome.

## License

This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for details.

---

Feel free to suggest improvements or report issues in the [Issues](https://github.com/shenawy29/harmonia/issues) section.
