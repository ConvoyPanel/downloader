<div align="center">
  <h3 align="center">Convoy Downloader</h3>

  <p align="center">
    A tool for importing pre-made templates to your Proxmox node. (Default templates provided free of charge!)
  </p>
</div>

## About The Project

Convoy downloader is a utility created by Performave to help users import pre-made templates to their Proxmox node. This
tool is designed to be used with Convoy Panel, but it can be used standalone too.

### Sponsor Us

There are default templates provided free of charge, but we encourage users
to [donate](https://donate.stripe.com/dR6dRIgbafyIaek000) because CDNs aren't free. You can find a list of the default
templates [here](https://images.cdn.convoypanel.com/images.json).

## Usage

1. Download the binary onto your Proxmox node for the corresponding architecture from
   the [releases](https://github.com/ConvoyPanel/downloader/releases/latest) page.
2. Make the binary executable (e.g., `chmod +x downloader_x86`)
3. Run the binary from the terminal (e.g., `./downloader_x86`)

### Custom Images List

You can specify a custom images list by providing the URL as an argument when running the binary. For example:

```sh
./downloader_x86 https://images.cdn.convoypanel.com/images.json
```

To understand the structure of the JSON file, you can view the default images
list [here](https://images.cdn.convoypanel.com/images.json).

## License

Distributed under the MIT License. See `LICENSE` for more information.