# Proxy Game for private game
So when we click `Start Game` it will run the proxy, which will direct all the domains in the list to `ps.yuuki.me`
private static bool HostPrivate(string hostname)
{
    if (
        hostname.EndsWith(".zenlesszonezero.com") |
        hostname.EndsWith(".honkaiimpact3.com") |
        hostname.EndsWith(".bhsr.com") |
        hostname.EndsWith(".starrails.com") |
        hostname.EndsWith(".yuanshen.com") |
        hostname.EndsWith(".hoyoverse.com") |
        hostname.EndsWith(".mihoyo.com"))
    {
        return true;
    }
    return false;
} this example c# code but we need to add this code to our launcher