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

// Kiểu BCL hay bị quên `using` trong khoá này. Dùng để phân biệt "thiếu
// using" (phải báo) với "tên do bài lược bỏ bối cảnh" (bỏ qua).
var bclTypeNames = new HashSet<string>(StringComparer.Ordinal)
{
    "Stopwatch", "Encoding", "Regex", "CultureInfo", "JsonSerializer",
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

int khoi = 0, khoiLoi = 0;
var thieuTen = new SortedDictionary<string, int>(StringComparer.Ordinal);
var khoiSai = new List<string>();

/// Biên dịch một khối, trả về chẩn đoán thuộc riêng khối đó.
List<Diagnostic> BienDich(string code, out SyntaxTree tree)
{
    tree = CSharpSyntaxTree.ParseText(code, parseOptions, path: "Program.cs");
    var t = tree;
    var comp = CSharpCompilation.Create(
        "Snippet",
        new[] { t, stubTree, usingTree },
        refs,
        new CSharpCompilationOptions(
            OutputKind.ConsoleApplication,
            allowUnsafe: false,
            nullableContextOptions: NullableContextOptions.Annotations));

    return comp.GetDiagnostics().Where(d => d.Location.SourceTree == t).ToList();
}

void Kiem(string nhan, string code, int dongTrongFile)
{
    khoi++;
    var all = BienDich(code, out var tree);
    int lechDong = 0;

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
        if (HuaCompilerChan(code) && loi.Count == 0)
            khoiSai.Add($"{nhan} (dòng {dongTrongFile}): comment hứa compiler chặn, nhưng khối biên dịch sạch");
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

    foreach (Match m in fence.Matches(text))
        Kiem(ten, m.Groups[1].Value, text[..m.Index].Count(c => c == '\n') + 1);

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
                    Kiem($"{ten} (câu hỏi {i})", c.GetString() ?? "", text[..mq.Index].Count(ch => ch == '\n') + 1);
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

Console.WriteLine($"\n{khoi} khối code, {khoiLoi} khối không biên dịch được.");
return khoiLoi == 0 ? 0 : 1;
