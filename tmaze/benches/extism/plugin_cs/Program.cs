using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text.Json;
using Extism;

namespace ExtismPlugin;

public class Program
{
    [UnmanagedCallersOnly(EntryPoint = "greet")]
    public static int Greet()
    {
        var name = Pdk.GetInputString();
        var greeting = $"Hello, {name}!";
        Pdk.SetOutput(greeting);

        return 0;
    }

    [UnmanagedCallersOnly(EntryPoint = "rf_ascii_buffer")]
    public static int RfAsciiBuffer()
    {
        var res64 = Pdk.GetInput();
        var w = BitConverter.ToInt32(res64.AsSpan()[..4]);
        var h = BitConverter.ToInt32(res64.AsSpan()[4..]);
        
        var buffer = new byte[w * h];
        
        for (var i = 0; i < buffer.Length; i++) buffer[i] = (byte)(i % 256);

        Pdk.SetOutput(buffer);

        return 0;
    }

    [UnmanagedCallersOnly(EntryPoint = "rf_mem_offset")]
    public static int RfMemOffset()
    {
        var res64 = Pdk.GetInput();
        var w = BitConverter.ToInt32(res64.AsSpan()[..4]);
        var h = BitConverter.ToInt32(res64.AsSpan()[4..]);
        
        var fbName = $"fb_{w}x{h}";

        if (Pdk.TryGetVar(fbName, out var buffer))
        {
            var arr = buffer.ReadBytes();
            Debug.Assert(arr.Length == w * h);
            
            for (var i = 0; i < arr.Length; i++) arr[i] = (byte)(i % 256);
            
            buffer.WriteBytes(arr);
            
            Pdk.SetOutput(buffer);
        }
        else
        {
            var arr = new byte[w * h];
            for (var i = 0; i < arr.Length; i++) arr[i] = (byte)(i % 256);

            var newBuf = Pdk.Allocate(arr);
            
            Pdk.SetVar(fbName, newBuf);
            
            Pdk.SetOutput(newBuf);
        }

        return 0;
    }

    // Note: a `Main` method is required for the app to compile
    public static void Main() {}
}
