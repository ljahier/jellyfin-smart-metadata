using MediaBrowser.Controller.Entities.Movies;
using MediaBrowser.Controller.Library;
using MediaBrowser.Controller.Providers;
using MediaBrowser.Model.Entities;
using Microsoft.Extensions.Logging;

namespace Jellyfin.Plugin.SmartMetadata;

public sealed class NormalizeMovieNameProvider : ICustomMetadataProvider<Movie>
{
    private readonly IProviderManager _providerManager;

    private readonly ILogger<NormalizeMovieNameProvider> _logger;

    public NormalizeMovieNameProvider(IProviderManager providerManager, ILogger<NormalizeMovieNameProvider> logger)
    {
        _providerManager = providerManager;
        _logger = logger;
    }

    public string Name => "Smart Metadata";

    public Task<ItemUpdateType> FetchAsync(Movie item, MetadataRefreshOptions options, CancellationToken cancellationToken)
    {
        if (!IsUnidentified(item))
        {
            return Task.FromResult(ItemUpdateType.None);
        }

        ParsedMovie? parsed;
        try
        {
            parsed = FilenameParser.Parse(Path.GetFileName(item.Path));
        }
        catch (Exception exception) when (exception is DllNotFoundException or EntryPointNotFoundException or PlatformNotSupportedException)
        {
            _logger.LogWarning(exception, "The Smart Metadata native library is unavailable.");
            return Task.FromResult(ItemUpdateType.None);
        }

        if (parsed is not { } movie || string.Equals(movie.Title, item.Name, StringComparison.Ordinal))
        {
            return Task.FromResult(ItemUpdateType.None);
        }

        _logger.LogInformation("Normalizing the name of {Path} to {Title}.", item.Path, movie.Title);
        item.Name = movie.Title;
        if (item.ProductionYear is null && movie.Year is { } year)
        {
            item.ProductionYear = year;
        }

        _providerManager.QueueRefresh(
            item.Id,
            new MetadataRefreshOptions(options.DirectoryService)
            {
                MetadataRefreshMode = MetadataRefreshMode.FullRefresh,
                ImageRefreshMode = MetadataRefreshMode.FullRefresh,
                ReplaceAllMetadata = false,
                ReplaceAllImages = false,
            },
            RefreshPriority.Low);

        return Task.FromResult(ItemUpdateType.MetadataEdit);
    }

    private static bool IsUnidentified(Movie item)
    {
        return item.ProviderIds.Count == 0
            && !string.IsNullOrEmpty(item.Path)
            && !item.ExtraType.HasValue
            && !item.LockedFields.Contains(MetadataField.Name);
    }
}
