// Kiểm tra CÚ PHÁP mọi khối ```csharp trong bài học IT (content/it).
//
// Bài học đầy code mẫu mà không có gì canh: gõ thiếu dấu chấm phẩy hay sai tên
// biến thì vẫn build ra HTML đẹp, người học mới là người phát hiện. Công cụ này
// đọc thẳng file .md và bắt lỗi trước.
//
// Chỉ PARSE, không bind kiểu: code trong bài là mảnh rời, gọi tới `Order`,
// `user`, `db`… không có thật là chuyện bình thường. Thứ cần bắt là lỗi cú pháp.
//
// Chạy: npm run check-csharp   (cần .NET SDK; không nằm trong `npm test` để
// máy không cài .NET vẫn chạy được toàn bộ test của web.)
using System.Text.RegularExpressions;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;

var root = args.Length > 0 ? args[0] : Path.Combine(Directory.GetCurrentDirectory(), "content", "it");
if (!Directory.Exists(root))
{
    Console.Error.WriteLine($"Không thấy thư mục nội dung: {root}");
    return 2;
}

// Khối code rộng quá thì điện thoại phải cuộn ngang mới đọc hết — đo trên
// màn 390px: quá 56 ký tự là bắt đầu phải cuộn. Đây mới là CẢNH BÁO, chưa
// chặn build, vì các bài viết trước chuẩn này còn nợ khá nhiều dòng.
const int maxWidth = 56;

var fence = new Regex("```csharp\r?\n(.*?)```", RegexOptions.Singleline);

// Câu hỏi cũng mang code (trường "code" trong khối ```quiz), mà code ở đó
// cũng hiện trong một khối pre y như trong bài — nên cũng phải vừa bề ngang.
var quizFence = new Regex("```quiz\r?\n(.*?)```", RegexOptions.Singleline);

int blocks = 0, bad = 0, wide = 0;
var wideFiles = new Dictionary<string, int>();

void DemRong(string name, string code)
{
    var n = code.Split('\n').Count(l => l.TrimEnd().Length > maxWidth);
    if (n == 0) return;
    wide += n;
    wideFiles[name] = wideFiles.GetValueOrDefault(name) + n;
}

foreach (var file in Directory.GetFiles(root, "*.md", SearchOption.AllDirectories).OrderBy(f => f))
{
    var text = File.ReadAllText(file);

    foreach (Match mq in quizFence.Matches(text))
    {
        using var doc = System.Text.Json.JsonDocument.Parse(mq.Groups[1].Value);
        foreach (var item in doc.RootElement.EnumerateArray())
        {
            if (item.TryGetProperty("code", out var c))
                DemRong(Path.GetFileName(file) + " (câu hỏi)", c.GetString() ?? "");
        }
    }

    foreach (Match m in fence.Matches(text))
    {
        blocks++;
        var code = m.Groups[1].Value;
        // Script mode cho phép trộn câu lệnh với khai báo class/method — đúng
        // kiểu mảnh code trong bài. Mảnh nào khai báo namespace thì phải đọc
        // như một file bình thường, script không cho phép namespace.
        var kind = code.Contains("namespace ") ? SourceCodeKind.Regular : SourceCodeKind.Script;
        var tree = CSharpSyntaxTree.ParseText(
            code,
            new CSharpParseOptions(LanguageVersion.Preview, kind: kind));

        // Script mode hiểu `using` ở đầu khối là using-directive, nên mảnh bắt
        // đầu bằng `using var x = …` hay có `await` sẽ báo lỗi oan. Thử lại
        // bằng cách bọc vào một method async rồi mới kết luận.
        if (tree.GetDiagnostics().Any(d => d.Severity == DiagnosticSeverity.Error))
        {
            var boc =
                "class __W { async System.Threading.Tasks.Task __M() {\n"
                + code
                + "\n} }";
            var treeBoc = CSharpSyntaxTree.ParseText(
                boc,
                new CSharpParseOptions(LanguageVersion.Preview));
            if (!treeBoc.GetDiagnostics().Any(d => d.Severity == DiagnosticSeverity.Error))
                tree = treeBoc;
        }

        DemRong(Path.GetFileName(file), code);

        var errors = tree.GetDiagnostics().Where(d => d.Severity == DiagnosticSeverity.Error).ToList();
        if (errors.Count == 0) continue;

        bad++;
        var lineOfBlock = text[..m.Index].Count(c => c == '\n') + 1;
        Console.WriteLine($"\n{Path.GetFileName(file)} (khối bắt đầu ở dòng {lineOfBlock}):");
        foreach (var d in errors.Take(5))
        {
            var pos = d.Location.GetLineSpan().StartLinePosition;
            var line = code.Split('\n').ElementAtOrDefault(pos.Line)?.TrimEnd() ?? "";
            Console.WriteLine($"  dòng {pos.Line + 1}: {d.Id} {d.GetMessage()}");
            Console.WriteLine($"    | {line}");
        }
    }
}

if (wide > 0)
{
    Console.WriteLine($"\nCảnh báo: {wide} dòng code dài quá {maxWidth} ký tự (điện thoại phải cuộn ngang):");
    foreach (var (name, n) in wideFiles.OrderByDescending(p => p.Value))
        Console.WriteLine($"  {n,3}  {name}");
}

Console.WriteLine($"\n{blocks} khối C#, {bad} khối có lỗi cú pháp.");
return bad == 0 ? 0 : 1;
