# Smart Metadata

A Jellyfin plugin that identifies movies with unconventional file names.

Jellyfin can fail on names such as
`2.Finding.Nemo.2003.MULTI.2160p.BluRay.HDR.REMUX.HEVC.TRUEHD.Atmos-GROUP .mkv`.
This plugin extracts a clean title and year with a small Rust parser, then lets
Jellyfin's own metadata providers (such as TMDb) search again with the clean
name.

It only changes the title (and the year, when empty) of movies that have no
provider id and whose name is not locked, then queues one refresh. Media files
are never touched.

## Install

1. In the Jellyfin dashboard, add this plugin repository:
   `https://github.com/ljahier/jellyfin-smart-metadata/releases/latest/download/manifest.json`
2. Install **Smart Metadata** from the catalog.
3. Restart Jellyfin.

Built for Jellyfin 12.4. Prebuilt releases contain a Linux x86_64 native
library only.

## Build

Requires Rust, the .NET 10 SDK and [mise](https://mise.jdx.dev). The project is
built and tested with Rust nightly.

```sh
mise run package
mise ci
```

`package` builds `dist/smart-metadata_<version>.zip`. `ci` runs formatting,
lints and tests.

## Release

Bump the version in `plugin/SmartMetadata.csproj` and `plugin/meta.json`, then
push a tag `v<version>` (for example `v0.1.0.0`). CI builds the zip and the
catalog manifest and publishes both to a GitHub release.

## License

[MPL-2.0](LICENSE)
