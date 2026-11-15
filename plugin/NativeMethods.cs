using System.Reflection;
using System.Runtime.InteropServices;

namespace Jellyfin.Plugin.SmartMetadata;

[StructLayout(LayoutKind.Sequential)]
internal struct NativeMovie
{
    public ushort Year;

    public nuint TitleLength;
}

internal static partial class NativeMethods
{
    private const string LibraryName = "jellyfin_smart_metadata";

    static NativeMethods()
    {
        NativeLibrary.SetDllImportResolver(typeof(NativeMethods).Assembly, ResolveLibrary);
    }

    [LibraryImport(LibraryName, EntryPoint = "sm_parse_movie_filename")]
    internal static partial int ParseMovieFilename(
        ReadOnlySpan<byte> fileName,
        nuint fileNameLength,
        Span<byte> titleBuffer,
        nuint titleBufferLength,
        out NativeMovie movie);

    private static IntPtr ResolveLibrary(string libraryName, Assembly assembly, DllImportSearchPath? searchPath)
    {
        if (libraryName != LibraryName)
        {
            return IntPtr.Zero;
        }

        var directory = Path.GetDirectoryName(assembly.Location)
            ?? throw new DllNotFoundException("The Smart Metadata plugin directory is unknown.");
        var path = Path.Combine(directory, LibraryFileName());

        return NativeLibrary.TryLoad(path, out var handle)
            ? handle
            : throw new DllNotFoundException($"The Smart Metadata native library is missing or cannot be loaded: {path}");
    }

    private static string LibraryFileName()
    {
        var architecture = RuntimeInformation.ProcessArchitecture switch
        {
            Architecture.X64 => "x64",
            Architecture.Arm64 => "arm64",
            _ => throw new PlatformNotSupportedException("Smart Metadata does not support this processor architecture."),
        };

        if (OperatingSystem.IsLinux())
        {
            return $"{LibraryName}-linux-{architecture}.so";
        }

        if (OperatingSystem.IsMacOS())
        {
            return $"{LibraryName}-osx-{architecture}.dylib";
        }

        if (OperatingSystem.IsWindows())
        {
            return $"{LibraryName}-win-{architecture}.dll";
        }

        throw new PlatformNotSupportedException("Smart Metadata does not support this operating system.");
    }
}
