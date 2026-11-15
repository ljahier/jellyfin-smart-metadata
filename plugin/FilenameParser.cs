using System.Text;

namespace Jellyfin.Plugin.SmartMetadata;

internal static class FilenameParser
{
    private const int FileNameBytesMax = 4096;

    private const int StatusSuccess = 0;

    public static ParsedMovie? Parse(string fileName)
    {
        var input = Encoding.UTF8.GetBytes(fileName);
        Span<byte> titleBuffer = stackalloc byte[FileNameBytesMax];

        var status = NativeMethods.ParseMovieFilename(
            input,
            (nuint)input.Length,
            titleBuffer,
            (nuint)titleBuffer.Length,
            out var movie);
        if (status != StatusSuccess)
        {
            return null;
        }

        var title = Encoding.UTF8.GetString(titleBuffer[..(int)movie.TitleLength]);
        return new ParsedMovie(title, movie.Year == 0 ? null : movie.Year);
    }
}
