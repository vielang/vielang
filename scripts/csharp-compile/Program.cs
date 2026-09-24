// BIÊN DỊCH THẬT mọi khối ```csharp và mọi trường "code" trong ```quiz.
//
// Vì sao có công cụ này, dù đã có check-csharp: công cụ kia chỉ PARSE, và
// parse ở Script mode — chế độ cho phép trộn câu lệnh với khai báo class.
// Nên nó bỏ lọt đúng những lỗi mà người học gặp khi dán code vào Program.cs:
//
//   CS8803  khai báo class đứng trước câu lệnh top-level
//   CS0160  bắt exception cha trước con — khối con thành khối chết
//   CS1061  gọi property không tồn tại (o.Items)
//   CS1503  items[..3] trên List<T> (Range chỉ chạy trên array/string/Span)
//   CS1998  method async mà bên trong không có await
//   CS0103  dùng Stopwatch mà thiếu using System.Diagnostics
//
// Cách làm: mỗi khối được ghi ra thành một Program.cs y nguyên, biên dịch
// cùng Stubs.cs (bối cảnh giả: db, logger, Order…). Giữ nguyên thứ tự trong
// khối là điều kiện bắt buộc — đó chính là cách bắt CS8803.
//
// Chạy: npm run check-code
using System.Collections.Immutable;
using System.Reflection;
using System.Text.Json;
using System.Text.RegularExpressions;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;

// Thông báo của Roslyn đi theo UICulture của máy. Máy dev đặt tiếng Hàn thì
// báo cáo ra tiếng Hàn — chốt về tiếng Anh để thông báo luôn tra được.
System.Globalization.CultureInfo.CurrentUICulture = new System.Globalization.CultureInfo("en-US");

var root = args.Length > 0 ? args[0] : Path.Combine(Directory.GetCurrentDirectory(), "content", "it");
if (!Directory.Exists(root))
{
    Console.Error.WriteLine($"Không thấy thư mục nội dung: {root}");
    return 2;
}

var stubPath = Path.Combine(AppContext.BaseDirectory, "Stubs.cs");
if (!File.Exists(stubPath))
{
    Console.Error.WriteLine($"Không thấy Stubs.cs cạnh file thực thi: {stubPath}");
    return 2;
}

var parseOptions = new CSharpParseOptions(LanguageVersion.Preview);
var stubTree = CSharpSyntaxTree.ParseText(File.ReadAllText(stubPath), parseOptions, path: "Stubs.cs");

// Global usings đúng như một console app .NET 9 bật ImplicitUsings: chỉ những
// namespace ấy, không thêm. Nhờ vậy khối quên `using System.Diagnostics` vẫn
// đỏ, y như lúc người học dán vào project của họ.
var usingTree = CSharpSyntaxTree.ParseText("""
    global using System;
    global using System.Collections.Generic;
    global using System.IO;
    global using System.Linq;
    global using System.Net.Http;
    global using System.Threading;
    global using System.Threading.Tasks;
    global using CourseWorld;
    global using static CourseWorld.Stubs;
    """, parseOptions, path: "GlobalUsings.cs");

// Tham chiếu toàn bộ framework đang chạy: đủ cho mọi khối trong khoá, và
// cũng là cách biết một tên như Stopwatch có thật trong BCL hay không.
var frameworkDir = Path.GetDirectoryName(typeof(object).Assembly.Location)!;
var refs = Directory.GetFiles(frameworkDir, "System*.dll")
    .Concat(new[] { Path.Combine(frameworkDir, "mscorlib.dll"), Path.Combine(frameworkDir, "netstandard.dll") })
    .Where(File.Exists)
    .Select(p => MetadataReference.CreateFromFile(p))
    .ToImmutableArray();

// Khoá ASP.NET Core là project web thật: nạp thêm shared framework của ASP.NET
// Core (FrameworkReference trong csproj), EF Core và xUnit (PackageReference,
// dll nằm cạnh file thực thi). Không nạp Stubs.cs — bối cảnh giả ấy viết cho
// khoá console, và các tên như ILogger của nó sẽ đụng với kiểu thật.
var aspNetDir = Path.GetDirectoryName(
    typeof(Microsoft.AspNetCore.Mvc.ControllerBase).Assembly.Location)!;
var webRefs = refs
    .Concat(Directory.GetFiles(aspNetDir, "Microsoft.*.dll")
        .Select(p => MetadataReference.CreateFromFile(p)))
    .Concat(Directory.GetFiles(AppContext.BaseDirectory, "*.dll")
        .Where(p => Path.GetFileName(p) is var f
            && (f.StartsWith("Microsoft.EntityFrameworkCore")
                || f.StartsWith("Oracle.")
                || f.StartsWith("EFCore.NamingConventions")
                || f.StartsWith("xunit")
                || f.StartsWith("Microsoft.AspNetCore.Authentication.JwtBearer")
                || f.StartsWith("Microsoft.IdentityModel")
                || f.StartsWith("System.IdentityModel")))
        .Select(p => MetadataReference.CreateFromFile(p)))
    .ToImmutableArray();

// Global usings đúng như project `dotnet new webapi` (Web SDK).
var webUsingTree = CSharpSyntaxTree.ParseText("""
    global using System;
    global using System.Collections.Generic;
    global using System.IO;
    global using System.Linq;
    global using System.Net.Http;
    global using System.Net.Http.Json;
    global using System.Threading;
    global using System.Threading.Tasks;
    global using Microsoft.AspNetCore.Builder;
    global using Microsoft.AspNetCore.Hosting;
    global using Microsoft.AspNetCore.Http;
    global using Microsoft.AspNetCore.Routing;
    global using Microsoft.Extensions.Configuration;
    global using Microsoft.Extensions.DependencyInjection;
    global using Microsoft.Extensions.Hosting;
    global using Microsoft.Extensions.Logging;
    """, parseOptions, path: "GlobalUsings.cs");

// Khoá WinForms: project `dotnet new winforms` (net9.0-windows). Nạp thêm
// dll Windows Forms từ shared framework Microsoft.WindowsDesktop.App, cùng EF
// Core và Oracle như khoá web. Bỏ các file trùng tên với BCL đã nạp.
var desktopRoot = Path.Combine(
    Path.GetDirectoryName(Path.GetDirectoryName(frameworkDir)!)!,
    "Microsoft.WindowsDesktop.App");
var phienBan = Path.GetFileName(frameworkDir);
var desktopDir = Directory.Exists(Path.Combine(desktopRoot, phienBan))
    ? Path.Combine(desktopRoot, phienBan)
    : Directory.Exists(desktopRoot)
        ? Directory.GetDirectories(desktopRoot)
            .Where(d => Path.GetFileName(d).Split('.')[0] == phienBan.Split('.')[0])
            .OrderBy(d => Version.Parse(Path.GetFileName(d).Split('-')[0]))
            .LastOrDefault()
        : null;
var tenDaNap = new HashSet<string>(
    Directory.GetFiles(frameworkDir, "*.dll").Select(Path.GetFileName)!,
    StringComparer.OrdinalIgnoreCase);
var winFormsRefs = desktopDir == null
    ? webRefs
    : webRefs
        .Concat(Directory.GetFiles(desktopDir, "*.dll")
            .Where(f => Path.GetFileName(f) is var n
                && !tenDaNap.Contains(n)
                && (n.StartsWith("System.Windows.Forms")
                    || n.StartsWith("System.Drawing")
                    || n.StartsWith("System.Private.Windows")
                    || n.StartsWith("Microsoft.Win32.SystemEvents")
                    || n.StartsWith("Accessibility")))
            .Select(p => MetadataReference.CreateFromFile(p)))
        .ToImmutableArray();

// Global usings của project WinForms (ImplicitUsings + UseWindowsForms), và
// ApplicationConfiguration mà WinForms SDK tự sinh trong namespace gốc.
var winFormsUsingTree = CSharpSyntaxTree.ParseText("""
    global using System;
    global using System.Collections.Generic;
    global using System.Drawing;
    global using System.IO;
    global using System.Linq;
    global using System.Net.Http;
    global using System.Threading;
    global using System.Threading.Tasks;
    global using System.Windows.Forms;

    namespace ShopDesk
    {
        internal static class ApplicationConfiguration
        {
            public static void Initialize() { }
        }
    }
    """, parseOptions, path: "GlobalUsings.cs");

// Đổi theo từng file: bài thuộc khoá aspnet-core thì biên dịch như project web.
var cheDoWeb = false;
var cheDoWinForms = false;

// Kiểu BCL hay bị quên `using` trong khoá này. Dùng để phân biệt "thiếu
// using" (phải báo) với "tên do bài lược bỏ bối cảnh" (bỏ qua).
var bclTypeNames = new HashSet<string>(StringComparer.Ordinal)
{
    "Stopwatch", "Encoding", "Regex", "CultureInfo", "JsonSerializer",
    // Khoá ASP.NET Core: quên `using Microsoft.AspNetCore.Mvc` hay
    // `using Microsoft.EntityFrameworkCore` là lỗi người học gặp thật.
    "ControllerBase", "ApiController", "ApiControllerAttribute",
    "RouteAttribute", "HttpGetAttribute", "HttpPostAttribute",
    "HttpPutAttribute", "HttpDeleteAttribute", "FromBodyAttribute",
    "FromQueryAttribute", "ActionResult", "IActionResult",
    "DbContext", "DbSet<>", "DbContextOptions<>", "ModelBuilder",
    "RequiredAttribute", "RangeAttribute", "StringLengthAttribute",
    "FactAttribute", "Assert",
};

var fence = new Regex("```csharp[^\n]*\r?\n(.*?)```", RegexOptions.Singleline);
var quizFence = new Regex("```quiz\r?\n(.*?)```", RegexOptions.Singleline);

// Lỗi không tính: tên do bài cố tình lược bỏ bối cảnh.
// CS0246 kiểu chưa khai báo, CS0103 tên chưa khai báo — trừ khi tên đó là
// một kiểu BCL, lúc ấy nghĩa là thiếu using và phải báo.
bool LaLoiOan(Diagnostic d)
{
    // Mảnh rời không có ngữ cảnh: `return NotFound();` là thân của một action,
    // `override` không biết lớp cha nào. Không kết luận được từ một khối.
    if (d.Id is "CS0161" or "CS0127" or "CS0115" or "CS0106") return true;

    if (d.Id is not ("CS0103" or "CS0246")) return false;
    var msg = d.GetMessage();
    return !bclTypeNames.Any(t => msg.Contains($"'{t}'"));
}

/// Khối cố tình sai để làm phản ví dụ. Phần lớn sai ở thời gian chạy hoặc sai
/// về thiết kế, nên biên dịch sạch là chuyện bình thường — không báo.
bool LaKhoiSai(string code) => code.Contains("// SAI");

/// Nhưng nếu chính comment trong khối hứa rằng compiler sẽ chặn, thì khối ấy
/// buộc phải lỗi. Biên dịch sạch nghĩa là bài đang dạy sai.
bool HuaCompilerChan(string code) =>
    code.Contains("lỗi compile") || code.Contains("compiler chặn")
    || code.Contains("không biên dịch") || code.Contains("KHÔNG biên dịch");

int khoi = 0, khoiLoi = 0, khoiMinhHoa = 0;
var thieuTen = new SortedDictionary<string, int>(StringComparer.Ordinal);
var khoiSai = new List<string>();

// Kiểu mà các khối TRƯỚC trong cùng bài đã khai báo. Bài thường dựng model
// riêng ở khối đầu (`record Order(string Customer, decimal Total)`) rồi dùng
// suốt các khối sau. Không mang theo thì khối sau bị soi bằng model chung
// trong Stubs.cs và báo lỗi oan.
var kieuTrongBai = new Dictionary<string, string>(StringComparer.Ordinal);

// Using của các khối trước, mang theo cùng kiểu: `ShopDbContext : DbContext`
// cần `using Microsoft.EntityFrameworkCore` của khối đã khai báo nó.
var usingTrongBai = new SortedSet<string>(StringComparer.Ordinal);

// Đã thử mang theo cả BIẾN của khối trước, và bỏ: các bài dùng lại tên `a`,
// `x`, `success` ở nhiều khối với nghĩa khác nhau, nên ghép vào là CS0128
// hàng loạt. Kiểu thì khác — một bài chỉ định nghĩa `Order` đúng một lần.

/// Biên dịch một khối, trả về chẩn đoán thuộc riêng khối đó.
List<Diagnostic> BienDich(string code, out SyntaxTree tree)
{
    tree = CSharpSyntaxTree.ParseText(code, parseOptions, path: "Program.cs");
    var t = tree;

    var daKhaiBao = new HashSet<string>(
        t.GetRoot().DescendantNodes().OfType<Microsoft.CodeAnalysis.CSharp.Syntax.BaseTypeDeclarationSyntax>()
            .Select(n => n.Identifier.Text),
        StringComparer.Ordinal);

    var cay = cheDoWinForms
        ? new List<SyntaxTree> { t, winFormsUsingTree }
        : cheDoWeb
        ? new List<SyntaxTree> { t, webUsingTree }
        : new List<SyntaxTree> { t, stubTree, usingTree };
    var boSung = kieuTrongBai.Where(p => !daKhaiBao.Contains(p.Key)).Select(p => p.Value).ToList();
    if (boSung.Count > 0)
        cay.Add(CSharpSyntaxTree.ParseText(
            string.Join("\n", usingTrongBai) + "\n\n" + string.Join("\n\n", boSung),
            parseOptions, path: "TrongBai.cs"));

    var comp = CSharpCompilation.Create(
        "Snippet",
        cay,
        cheDoWinForms ? winFormsRefs : cheDoWeb ? webRefs : refs,
        new CSharpCompilationOptions(
            OutputKind.ConsoleApplication,
            allowUnsafe: false,
            nullableContextOptions: NullableContextOptions.Annotations));

    return comp.GetDiagnostics().Where(d => d.Location.SourceTree == t).ToList();
}

void Kiem(string nhan, string code, int dongTrongFile, bool nghiemNgat = true)
{
    khoi++;
    var all = BienDich(code, out var tree);
    int lechDong = 0;

    // Ghi lại kiểu và biến bài vừa khai báo, cho các khối sau dùng.
    foreach (var kieu in tree.GetRoot().DescendantNodes()
                 .OfType<Microsoft.CodeAnalysis.CSharp.Syntax.BaseTypeDeclarationSyntax>())
        kieuTrongBai.TryAdd(kieu.Identifier.Text, kieu.ToFullString());
    foreach (var u in tree.GetRoot().DescendantNodes()
                 .OfType<Microsoft.CodeAnalysis.CSharp.Syntax.UsingDirectiveSyntax>())
        usingTrongBai.Add(u.ToString());


    // Nhiều khối là THÂN CLASS trình bày rời: `public Task<Order> GetAsync(…)`
    // đứng một mình. Ở file top-level, `public` trên một local function là
    // CS0106 — lỗi oan, vì người học sẽ dán nó vào trong một class.
    // Thử lại bằng cách bọc vào class. Nhưng chỉ thử khi KHÔNG có CS8803:
    // CS8803 nghĩa là khối trộn sai thứ tự khai báo với câu lệnh, và đó là
    // lỗi thật phải giữ.
    var laThanClass = all.Any(d => d.Id is "CS0106" or "CS0116" or "CS1525" or "CS9348")
        && !all.Any(d => d.Id == "CS8803");
    if (laThanClass)
    {
        var boc = BienDich("class __Wrap {\n" + code + "\n}", out var treeBoc);
        if (boc.Count(d => d.Severity == DiagnosticSeverity.Error)
            < all.Count(d => d.Severity == DiagnosticSeverity.Error))
        {
            all = boc;
            tree = treeBoc;
            lechDong = -1;
        }
    }

    foreach (var d in all.Where(LaLoiOan))
    {
        var m = Regex.Match(d.GetMessage(), "'([^']+)'");
        if (m.Success) thieuTen[m.Groups[1].Value] = thieuTen.GetValueOrDefault(m.Groups[1].Value) + 1;
    }

    var loi = all
        .Where(d => d.Severity == DiagnosticSeverity.Error && !LaLoiOan(d))
        .Concat(all.Where(d => d.Id is "CS1998" or "CS0162" or "CS0168"))
        .DistinctBy(d => (d.Id, d.Location.GetLineSpan().StartLinePosition.Line))
        .OrderBy(d => d.Location.GetLineSpan().StartLinePosition.Line)
        .ToList();

    // Khối phản ví dụ: lỗi là đúng ý. Chỉ cần nó lỗi VÌ ĐÚNG LÝ DO, nên in ra
    // mã lỗi để tác giả đối chiếu với lời mình viết.
    if (LaKhoiSai(code) || HuaCompilerChan(code))
    {
        // Khối đã hứa có lỗi thì mọi lỗi thật đều tính, kể cả loại thường bị
        // coi là "oan" với mảnh rời (CS0161 thiếu return là chính điều bài dạy).
        if (HuaCompilerChan(code) && !all.Any(d => d.Severity == DiagnosticSeverity.Error))
            khoiSai.Add($"{nhan} (dòng {dongTrongFile}): comment hứa compiler chặn, nhưng khối biên dịch sạch");
        return;
    }

    // Khối minh hoạ được phép viết khai báo trước rồi mới dùng: đọc xuôi hơn,
    // và giống cách dự án thật đặt kiểu ở file riêng. Chỉ mục "Thử ngay" mới
    // buộc dán thẳng vào Program.cs là chạy, nên mới xét CS8803.
    if (!nghiemNgat && loi.Count > 0 && loi.All(d => d.Id == "CS8803"))
    {
        khoiMinhHoa++;
        return;
    }

    if (loi.Count == 0) return;

    khoiLoi++;
    Console.WriteLine($"\n{nhan} (khối ở dòng {dongTrongFile}):");
    var dong = code.Split('\n');
    foreach (var d in loi.Take(4))
    {
        var i = d.Location.GetLineSpan().StartLinePosition.Line + lechDong;
        Console.WriteLine($"  {d.Id} dòng {i + 1}: {d.GetMessage()}");
        Console.WriteLine($"    | {dong.ElementAtOrDefault(i)?.TrimEnd()}");
    }
}

foreach (var file in Directory.GetFiles(root, "*.md", SearchOption.AllDirectories).OrderBy(f => f))
{
    if (Path.GetFileName(file) is "FORMAT.md") continue;
    var text = File.ReadAllText(file);
    var ten = Path.GetFileName(file);
    kieuTrongBai.Clear();   // mỗi bài một thế giới riêng
    usingTrongBai.Clear();
    // Khoá SQL cũng cần tham chiếu Oracle (Oracle.ManagedDataAccess) nên dùng
    // chung bộ tham chiếu với khoá ASP.NET Core.
    var duongDan = file.Replace('\\', '/');
    cheDoWeb = duongDan.Contains("/aspnet-core/") || duongDan.Contains("/sql/");
    cheDoWinForms = duongDan.Contains("/winforms/");

    foreach (Match m in fence.Matches(text))
    {
        // Mục chứa khối quyết định mức nghiêm ngặt.
        var truoc = text[..m.Index];
        var tieuDe = Regex.Matches(truoc, "^## (.+)$", RegexOptions.Multiline).LastOrDefault()?.Groups[1].Value ?? "";
        Kiem(ten, m.Groups[1].Value, truoc.Count(c => c == '\n') + 1,
            nghiemNgat: tieuDe.TrimStart().StartsWith("Thử ngay"));
    }

    foreach (Match mq in quizFence.Matches(text))
    {
        JsonDocument doc;
        try { doc = JsonDocument.Parse(mq.Groups[1].Value); }
        catch (JsonException e) { Console.WriteLine($"\n{ten}: quiz không phải JSON hợp lệ — {e.Message}"); khoiLoi++; continue; }
        using (doc)
        {
            int i = 0;
            foreach (var item in doc.RootElement.EnumerateArray())
            {
                i++;
                if (item.TryGetProperty("code", out var c))
                    // Code trong câu hỏi là để ĐỌC chứ không phải để dán chạy.
                    Kiem($"{ten} (câu hỏi {i})", c.GetString() ?? "",
                        text[..mq.Index].Count(ch => ch == '\n') + 1, nghiemNgat: false);
            }
        }
    }
}

if (thieuTen.Count > 0)
{
    Console.WriteLine($"\nTên bài giả định có sẵn (đã bỏ qua) — thêm vào Stubs.cs nếu muốn soi kỹ hơn:");
    foreach (var (ten, n) in thieuTen.OrderByDescending(p => p.Value).Take(25))
        Console.WriteLine($"  {n,3}  {ten}");
}

if (khoiSai.Count > 0)
{
    Console.WriteLine("\nKhối phản ví dụ (lỗi là đúng ý) — đối chiếu mã lỗi với lời bài:");
    foreach (var d in khoiSai) Console.WriteLine($"  {d}");
}

if (khoiMinhHoa > 0)
    Console.WriteLine($"\n{khoiMinhHoa} khối minh hoạ khai báo trước rồi mới dùng — hợp lệ, chỉ là không dán thẳng vào Program.cs được.");

Console.WriteLine($"\n{khoi} khối code, {khoiLoi} khối không biên dịch được.");
return khoiLoi == 0 ? 0 : 1;
