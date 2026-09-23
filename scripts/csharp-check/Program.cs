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

var fence = new Regex("```csharp\r?\n(.*?)```", RegexOptions.Singleline);
int blocks = 0, bad = 0;

foreach (var file in Directory.GetFiles(root, "*.md", SearchOption.AllDirectories).OrderBy(f => f))
{
    var text = File.ReadAllText(file);
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

Console.WriteLine($"\n{blocks} khối C#, {bad} khối có lỗi cú pháp.");
return bad == 0 ? 0 : 1;
