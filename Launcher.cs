using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.IO;
using System.Net;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using System.Windows.Forms;

[assembly: AssemblyTitle("MySession — Gestione Sessioni di Studio & Calendario Esami")]
[assembly: AssemblyDescription("Desktop Study Session & Exam Calendar — Multi-sessione & Calendario Infinito")]
[assembly: AssemblyCompany("MySession")]
[assembly: AssemblyProduct("MySession")]
[assembly: AssemblyVersion("3.1.0.0")]
[assembly: AssemblyFileVersion("3.1.0.0")]

namespace MySessionApp
{
    static class Program
    {
        private const int HTTP_PORT = 49271;
        private static IntPtr _sessionHwnd = IntPtr.Zero;
        private static HttpListener _listener = null;
        private static bool _isRunning = true;
        private static string _baseDir = "";
        private static string _currentFrameStyle = "custom";
        private static int _browserPid = 0;

        // Win32 API Imports (Compatibili 32-bit e 64-bit)
        [DllImport("user32.dll")]
        private static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
        private delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

        [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
        private static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);

        [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
        private static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);

        [DllImport("user32.dll")]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool IsWindowVisible(IntPtr hWnd);

        [DllImport("user32.dll")]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool IsWindow(IntPtr hWnd);

        [DllImport("user32.dll")]
        private static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);

        [DllImport("user32.dll", EntryPoint = "GetWindowLong", SetLastError = true)]
        private static extern int GetWindowLong32(IntPtr hWnd, int nIndex);

        [DllImport("user32.dll", EntryPoint = "GetWindowLongPtr", SetLastError = true)]
        private static extern IntPtr GetWindowLongPtr64(IntPtr hWnd, int nIndex);

        [DllImport("user32.dll", EntryPoint = "SetWindowLong", SetLastError = true)]
        private static extern int SetWindowLong32(IntPtr hWnd, int nIndex, int dwNewLong);

        [DllImport("user32.dll", EntryPoint = "SetWindowLongPtr", SetLastError = true)]
        private static extern IntPtr SetWindowLongPtr64(IntPtr hWnd, int nIndex, IntPtr dwNewLong);

        private static long GetWindowLongPtr(IntPtr hWnd, int nIndex)
        {
            if (IntPtr.Size == 8)
                return GetWindowLongPtr64(hWnd, nIndex).ToInt64();
            return GetWindowLong32(hWnd, nIndex);
        }

        private static long SetWindowLongPtr(IntPtr hWnd, int nIndex, long dwNewLong)
        {
            if (IntPtr.Size == 8)
                return SetWindowLongPtr64(hWnd, nIndex, new IntPtr(dwNewLong)).ToInt64();
            return SetWindowLong32(hWnd, nIndex, (int)dwNewLong);
        }

        [DllImport("user32.dll", SetLastError = true)]
        private static extern bool SetWindowPos(IntPtr hWnd, IntPtr hWndInsertAfter, int X, int Y, int cx, int cy, uint uFlags);

        [DllImport("user32.dll")]
        private static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

        [DllImport("user32.dll")]
        private static extern bool IsZoomed(IntPtr hWnd);

        [DllImport("user32.dll")]
        private static extern bool PostMessage(IntPtr hWnd, uint Msg, IntPtr wParam, IntPtr lParam);

        [DllImport("user32.dll")]
        private static extern bool SetForegroundWindow(IntPtr hWnd);

        [DllImport("dwmapi.dll")]
        private static extern int DwmExtendFrameIntoClientArea(IntPtr hWnd, ref MARGINS pMarInset);

        [StructLayout(LayoutKind.Sequential)]
        public struct MARGINS
        {
            public int cxLeftWidth;
            public int cxRightWidth;
            public int cyTopHeight;
            public int cyBottomHeight;
        }

        private const int GWL_STYLE = -16;
        private const long WS_CAPTION = 0x00C00000L;
        private const long WS_THICKFRAME = 0x00040000L;
        private const long WS_SYSMENU = 0x00080000L;
        private const long WS_MINIMIZEBOX = 0x00020000L;
        private const long WS_MAXIMIZEBOX = 0x00010000L;

        private const uint SWP_FRAMECHANGED = 0x0020;
        private const uint SWP_NOMOVE = 0x0002;
        private const uint SWP_NOSIZE = 0x0001;
        private const uint SWP_NOZORDER = 0x0004;
        private const uint SWP_NOACTIVATE = 0x0010;

        private const int SW_MINIMIZE = 6;
        private const int SW_MAXIMIZE = 3;
        private const int SW_RESTORE = 9;
        private const uint WM_CLOSE = 0x0010;

        [STAThread]
        static void Main(string[] args)
        {
            _baseDir = AppDomain.CurrentDomain.BaseDirectory;

            if (args.Length > 0 && args[0] == "--make-icon")
            {
                GenerateIcon(Path.Combine(_baseDir, "app.ico"));
                return;
            }

            // Se un'altra istanza di MySession è già attiva, portala in primo piano ed esci
            if (TryActivateExistingInstance())
            {
                return;
            }

            string htmlPath = Path.Combine(_baseDir, "index.html");
            if (!File.Exists(htmlPath))
            {
                MessageBox.Show(
                    "File index.html non trovato nella cartella dell'applicazione:\n" + htmlPath,
                    "MySession",
                    MessageBoxButtons.OK,
                    MessageBoxIcon.Error
                );
                return;
            }

            // Leggi preferenza cornice salvata (default: custom)
            string prefFile = Path.Combine(_baseDir, "frame_style.txt");
            if (File.Exists(prefFile))
            {
                try
                {
                    string saved = File.ReadAllText(prefFile).Trim().ToLowerInvariant();
                    if (saved == "standard") _currentFrameStyle = "standard";
                    else _currentFrameStyle = "custom";
                }
                catch { }
            }

            string fileUri = new Uri(htmlPath).AbsoluteUri;
            string profileDir = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                "MySession_Desktop_AppProfile"
            );

            string[] browserCandidates = new string[]
            {
                @"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
                @"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
                @"C:\Program Files\Google\Chrome\Application\chrome.exe",
                @"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"
            };

            Process browserProc = null;
            foreach (string browserPath in browserCandidates)
            {
                if (File.Exists(browserPath))
                {
                    ProcessStartInfo psi = new ProcessStartInfo
                    {
                        FileName = browserPath,
                        Arguments = string.Format(
                            "--app=\"{0}\" --window-size=1440,920 --user-data-dir=\"{1}\" --no-first-run --no-default-browser-check --disable-extensions",
                            fileUri,
                            profileDir
                        ),
                        UseShellExecute = false
                    };
                    try
                    {
                        browserProc = Process.Start(psi);
                        if (browserProc != null)
                        {
                            try { _browserPid = browserProc.Id; } catch { }
                        }
                        break;
                    }
                    catch { }
                }
            }

            if (browserProc == null)
            {
                // Fallback al browser predefinito di sistema
                Process.Start(new ProcessStartInfo(htmlPath) { UseShellExecute = true });
                return;
            }

            // Avvia bridge HTTP locale per consentire all'interfaccia HTML di controllare cornice e finestra
            StartHttpBridge();

            // Thread di monitoraggio per agganciare l'HWND della finestra ed applicare lo stile cornice scelto
            Thread monitorThread = new Thread(new ThreadStart(MonitorWindowLoop));
            monitorThread.IsBackground = true;
            monitorThread.Start();

            // Mantieni il processo attivo fino alla chiusura dell'interfaccia
            while (_isRunning)
            {
                Thread.Sleep(500);
                if (_sessionHwnd != IntPtr.Zero && !IsWindow(_sessionHwnd))
                {
                    _isRunning = false;
                }
            }

            StopHttpBridge();
        }

        private static bool TryActivateExistingInstance()
        {
            try
            {
                HttpWebRequest req = (HttpWebRequest)WebRequest.Create(string.Format("http://127.0.0.1:{0}/api/activate", HTTP_PORT));
                req.Timeout = 800;
                req.Method = "GET";
                using (HttpWebResponse resp = (HttpWebResponse)req.GetResponse())
                {
                    if (resp.StatusCode == HttpStatusCode.OK)
                    {
                        return true;
                    }
                }
            }
            catch { }
            return false;
        }

        private static void MonitorWindowLoop()
        {
            // Fase 1: Ricerca iniziale e prima applicazione dello stile (fino a 10 secondi)
            int attempts = 0;
            while (_isRunning && attempts < 100)
            {
                if (_sessionHwnd == IntPtr.Zero || !IsWindow(_sessionHwnd))
                {
                    _sessionHwnd = FindMySessionWindow();
                }

                if (_sessionHwnd != IntPtr.Zero)
                {
                    ApplyWindowFrameStyleInternal(_currentFrameStyle);
                    // Una volta agganciata stabilmente l'applicazione per almeno 15 iterazioni, passa alla fase 2
                    if (attempts > 15) break;
                }

                Thread.Sleep(100);
                attempts++;
            }

            // Fase 2: Monitoraggio continuo in background
            // Assicura che la barra nativa di Windows non riappaia dopo redraw/navigazioni interne del browser
            while (_isRunning)
            {
                Thread.Sleep(400);
                if (_sessionHwnd == IntPtr.Zero || !IsWindow(_sessionHwnd))
                {
                    _sessionHwnd = FindMySessionWindow();
                }

                if (_sessionHwnd != IntPtr.Zero && _currentFrameStyle == "custom")
                {
                    long curStyle = GetWindowLongPtr(_sessionHwnd, GWL_STYLE);
                    if ((curStyle & WS_CAPTION) != 0)
                    {
                        // Il browser ha ripristinato WS_CAPTION: riapplica la rimozione della barra nativa
                        ApplyWindowFrameStyleInternal("custom");
                    }
                }
            }
        }

        private static IntPtr FindMySessionWindow()
        {
            IntPtr found = IntPtr.Zero;
            EnumWindows(delegate(IntPtr hWnd, IntPtr lParam)
            {
                if (IsWindowVisible(hWnd))
                {
                    StringBuilder sbTitle = new StringBuilder(256);
                    GetWindowText(hWnd, sbTitle, 256);
                    StringBuilder sbClass = new StringBuilder(256);
                    GetClassName(hWnd, sbClass, 256);
                    string title = sbTitle.ToString();
                    string className = sbClass.ToString();

                    if (className == "Chrome_WidgetWin_1" || className.Contains("Chrome"))
                    {
                        uint pid = 0;
                        GetWindowThreadProcessId(hWnd, out pid);

                        bool matchPid = (_browserPid != 0 && pid == (uint)_browserPid);
                        bool matchTitle = (title.Contains("MySession") || title.Contains("Calendario") || title.Contains("Sessione"));

                        if (matchTitle || (matchPid && !string.IsNullOrEmpty(title)))
                        {
                            found = hWnd;
                            return false;
                        }
                    }
                }
                return true;
            }, IntPtr.Zero);
            return found;
        }

        public static void ApplyWindowFrameStyleInternal(string style)
        {
            if (_sessionHwnd == IntPtr.Zero || !IsWindow(_sessionHwnd))
            {
                _sessionHwnd = FindMySessionWindow();
            }
            if (_sessionHwnd == IntPtr.Zero || !IsWindow(_sessionHwnd)) return;

            _currentFrameStyle = (style == "standard") ? "standard" : "custom";

            if (_currentFrameStyle == "custom")
            {
                long curStyle = GetWindowLongPtr(_sessionHwnd, GWL_STYLE);
                // Rimuove completamente WS_CAPTION (0x00C00000 = WS_BORDER | WS_DLGFRAME)
                // Mantiene WS_THICKFRAME per consentire ridimensionamento e Aero Snap di Windows 11
                long newStyle = (curStyle & ~WS_CAPTION) | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;
                SetWindowLongPtr(_sessionHwnd, GWL_STYLE, newStyle);
                SetWindowPos(_sessionHwnd, IntPtr.Zero, 0, 0, 0, 0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED | SWP_NOACTIVATE);

                // Estende il frame DWM per mantenere l'ombreggiatura morbida e i bordi arrotondati di Windows 11
                try
                {
                    MARGINS margins = new MARGINS();
                    margins.cxLeftWidth = 1;
                    margins.cxRightWidth = 1;
                    margins.cyTopHeight = 1;
                    margins.cyBottomHeight = 1;
                    DwmExtendFrameIntoClientArea(_sessionHwnd, ref margins);
                }
                catch { }
            }
            else
            {
                long curStyle = GetWindowLongPtr(_sessionHwnd, GWL_STYLE);
                // Ripristina la barra del titolo classica e il menu di sistema nativo di Windows
                long newStyle = curStyle | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;
                SetWindowLongPtr(_sessionHwnd, GWL_STYLE, newStyle);
                SetWindowPos(_sessionHwnd, IntPtr.Zero, 0, 0, 0, 0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED | SWP_NOACTIVATE);
            }

            // Salva preferenza persistente su file per i successivi riavvii
            try
            {
                string prefFile = Path.Combine(_baseDir, "frame_style.txt");
                File.WriteAllText(prefFile, _currentFrameStyle);
            }
            catch { }
        }

        private static void StartHttpBridge()
        {
            try
            {
                _listener = new HttpListener();
                _listener.Prefixes.Add(string.Format("http://127.0.0.1:{0}/api/", HTTP_PORT));
                _listener.Start();
                ThreadPool.QueueUserWorkItem(delegate
                {
                    while (_isRunning && _listener != null && _listener.IsListening)
                    {
                        try
                        {
                            HttpListenerContext ctx = _listener.GetContext();
                            ProcessHttpRequest(ctx);
                        }
                        catch { }
                    }
                });
            }
            catch { }
        }

        private static void StopHttpBridge()
        {
            try
            {
                if (_listener != null)
                {
                    _listener.Stop();
                    _listener.Close();
                }
            }
            catch { }
        }

        private static void ProcessHttpRequest(HttpListenerContext ctx)
        {
            HttpListenerRequest req = ctx.Request;
            HttpListenerResponse res = ctx.Response;

            res.Headers.Add("Access-Control-Allow-Origin", "*");
            res.Headers.Add("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
            res.Headers.Add("Access-Control-Allow-Headers", "*");

            if (req.HttpMethod == "OPTIONS")
            {
                res.StatusCode = 204;
                res.Close();
                return;
            }

            string path = req.Url.AbsolutePath.ToLowerInvariant();
            string jsonResponse = "{\"status\":\"ok\"}";

            if (_sessionHwnd == IntPtr.Zero || !IsWindow(_sessionHwnd))
            {
                _sessionHwnd = FindMySessionWindow();
            }

            if (path.EndsWith("/frame"))
            {
                string style = req.QueryString["style"];
                if (!string.IsNullOrEmpty(style))
                {
                    ApplyWindowFrameStyleInternal(style);
                }
                jsonResponse = string.Format("{{\"status\":\"ok\",\"frame\":\"{0}\"}}", _currentFrameStyle);
            }
            else if (path.EndsWith("/minimize"))
            {
                if (_sessionHwnd != IntPtr.Zero)
                {
                    ShowWindow(_sessionHwnd, SW_MINIMIZE);
                }
                jsonResponse = "{\"status\":\"ok\",\"action\":\"minimize\"}";
            }
            else if (path.EndsWith("/maximize"))
            {
                bool isMax = false;
                if (_sessionHwnd != IntPtr.Zero)
                {
                    isMax = IsZoomed(_sessionHwnd);
                    ShowWindow(_sessionHwnd, isMax ? SW_RESTORE : SW_MAXIMIZE);
                    isMax = !isMax;
                    if (_currentFrameStyle == "custom")
                    {
                        ThreadPool.QueueUserWorkItem(delegate
                        {
                            Thread.Sleep(60);
                            ApplyWindowFrameStyleInternal("custom");
                        });
                    }
                }
                jsonResponse = string.Format("{{\"status\":\"ok\",\"isMaximized\":{0}}}", isMax ? "true" : "false");
            }
            else if (path.EndsWith("/close"))
            {
                if (_sessionHwnd != IntPtr.Zero)
                {
                    PostMessage(_sessionHwnd, WM_CLOSE, IntPtr.Zero, IntPtr.Zero);
                }
                _isRunning = false;
                jsonResponse = "{\"status\":\"ok\",\"action\":\"close\"}";
            }
            else if (path.EndsWith("/activate"))
            {
                if (_sessionHwnd != IntPtr.Zero)
                {
                    if (IsZoomed(_sessionHwnd)) ShowWindow(_sessionHwnd, SW_MAXIMIZE);
                    else ShowWindow(_sessionHwnd, SW_RESTORE);
                    SetForegroundWindow(_sessionHwnd);
                }
                jsonResponse = "{\"status\":\"ok\",\"action\":\"activate\"}";
            }
            else if (path.EndsWith("/ping"))
            {
                bool hasWin = (_sessionHwnd != IntPtr.Zero && IsWindow(_sessionHwnd));
                jsonResponse = string.Format("{{\"status\":\"ok\",\"hasWindow\":{0},\"frame\":\"{1}\"}}", hasWin ? "true" : "false", _currentFrameStyle);
            }

            byte[] buffer = Encoding.UTF8.GetBytes(jsonResponse);
            res.ContentType = "application/json";
            res.ContentLength64 = buffer.Length;
            res.OutputStream.Write(buffer, 0, buffer.Length);
            res.OutputStream.Close();
        }

        static void GenerateIcon(string outputPath)
        {
            using (Bitmap bmp = new Bitmap(64, 64))
            using (Graphics g = Graphics.FromImage(bmp))
            {
                g.SmoothingMode = SmoothingMode.AntiAlias;
                g.Clear(Color.Transparent);

                // Rounded Blueprint Tile (#00629B)
                using (GraphicsPath path = RoundedRect(new Rectangle(2, 2, 60, 60), 14))
                using (LinearGradientBrush bgBrush = new LinearGradientBrush(
                    new Rectangle(0, 0, 64, 64),
                    Color.FromArgb(0, 114, 178),
                    Color.FromArgb(0, 74, 119),
                    LinearGradientMode.ForwardDiagonal))
                {
                    g.FillPath(bgBrush, path);
                }

                // Top Calendar Accent Bar (Vermilion #D9381E)
                using (GraphicsPath topBar = RoundedRect(new Rectangle(10, 10, 44, 12), 4))
                using (SolidBrush redBrush = new SolidBrush(Color.FromArgb(217, 56, 30)))
                {
                    g.FillPath(redBrush, topBar);
                }

                // Calendar Card Body
                using (GraphicsPath cardBody = RoundedRect(new Rectangle(10, 20, 44, 34), 6))
                using (SolidBrush whiteBrush = new SolidBrush(Color.FromArgb(248, 250, 252)))
                {
                    g.FillPath(whiteBrush, cardBody);
                }

                // "MS" (MySession) Numeral / Emblem
                using (Font f = new Font("Segoe UI", 15f, FontStyle.Bold, GraphicsUnit.Pixel))
                using (SolidBrush textBrush = new SolidBrush(Color.FromArgb(15, 23, 42)))
                {
                    StringFormat sf = new StringFormat
                    {
                        Alignment = StringAlignment.Center,
                        LineAlignment = StringAlignment.Center
                    };
                    g.DrawString("MS", f, textBrush, new RectangleF(10, 22, 44, 30), sf);
                }

                // Salva file ICO compatibile 32-bit PNG
                using (MemoryStream pngStream = new MemoryStream())
                {
                    bmp.Save(pngStream, System.Drawing.Imaging.ImageFormat.Png);
                    byte[] pngBytes = pngStream.ToArray();

                    using (FileStream fs = new FileStream(outputPath, FileMode.Create))
                    using (BinaryWriter bw = new BinaryWriter(fs))
                    {
                        bw.Write((short)0);
                        bw.Write((short)1);
                        bw.Write((short)1);

                        bw.Write((byte)64);
                        bw.Write((byte)64);
                        bw.Write((byte)0);
                        bw.Write((byte)0);
                        bw.Write((short)1);
                        bw.Write((short)32);
                        bw.Write(pngBytes.Length);
                        bw.Write(22);

                        bw.Write(pngBytes);
                    }
                }
            }
        }

        static GraphicsPath RoundedRect(Rectangle bounds, int radius)
        {
            int d = radius * 2;
            GraphicsPath path = new GraphicsPath();
            path.AddArc(bounds.Left, bounds.Top, d, d, 180, 90);
            path.AddArc(bounds.Right - d, bounds.Top, d, d, 270, 90);
            path.AddArc(bounds.Right - d, bounds.Bottom - d, d, d, 0, 90);
            path.AddArc(bounds.Left, bounds.Bottom - d, d, d, 90, 90);
            path.CloseFigure();
            return path;
        }
    }
}
