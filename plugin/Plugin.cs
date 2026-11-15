using MediaBrowser.Common.Plugins;

namespace Jellyfin.Plugin.SmartMetadata;

public sealed class Plugin : BasePlugin
{
    private static readonly Guid PluginId = Guid.Parse("3f7a9c1e-5b24-4d8a-a6f0-92c1d7e84b35");

    public Plugin()
    {
        var assembly = typeof(Plugin).Assembly;
        var assemblyFilePath = assembly.Location;
        SetAttributes(
            assemblyFilePath,
            Path.GetDirectoryName(assemblyFilePath) ?? string.Empty,
            assembly.GetName().Version ?? new Version(0, 0, 0, 0));
        SetId(PluginId);
    }

    public override string Name => "Smart Metadata";
}
