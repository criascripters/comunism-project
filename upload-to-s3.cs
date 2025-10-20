using System;
using System.IO;
using System.Threading.Tasks;
using Amazon;
using Amazon.S3;
using Amazon.S3.Model;
using Amazon.S3.Transfer;
using Microsoft.Extensions.Configuration;

internal class Program
{
    private static async Task Main(string[] args)
    {
        Console.WriteLine("------------------------------------------------------------------");
        Console.WriteLine("Bem vindo ao SnorlaxS3, software para automatizar seus backups");
        Console.WriteLine("------------------------------------------------------------------");

        // ✅ Load AWS credentials from auth/aws-keys.json
        var configuration = new ConfigurationBuilder()
            .SetBasePath(AppDomain.CurrentDomain.BaseDirectory)
            .AddJsonFile(Path.Combine("auth", "aws-keys.json"), optional: false, reloadOnChange: true)
            .Build();

        string? accessKey = configuration["AppSettings:accesskey"];
        string? secretKey = configuration["AppSettings:secretkey"];

        if (string.IsNullOrEmpty(accessKey) || string.IsNullOrEmpty(secretKey))
        {
            Console.WriteLine("❌ Erro: Chaves AWS não encontradas no arquivo aws-keys.json.");
            return;
        }

        var credentials = new Amazon.Runtime.BasicAWSCredentials(accessKey, secretKey);
        var region = RegionEndpoint.USEast1;
        var s3Client = new AmazonS3Client(credentials, region);

        // ✅ List buckets
        var listResponse = await s3Client.ListBucketsAsync();
        Console.WriteLine($"Quantidade de buckets: {listResponse.Buckets.Count}");
        Console.WriteLine("------------------------------------------------------------------");
        Console.WriteLine("ID    |                  Nome");
        Console.WriteLine("------------------------------------------------------------------");

        int index = 1;
        foreach (var bucket in listResponse.Buckets)
        {
            Console.WriteLine($"ID {index}   Bucket name {bucket.BucketName}");
            index++;
        }

        Console.WriteLine("------------------------------------------------------------------");
        Console.Write("Escolha um bucket para fazer o upload de seus arquivos (use o ID): ");
        if (!int.TryParse(Console.ReadLine(), out int bucketChoice) || bucketChoice < 1 || bucketChoice > listResponse.Buckets.Count)
        {
            Console.WriteLine("❌ Escolha inválida.");
            return;
        }

        string bucketName = listResponse.Buckets[bucketChoice - 1].BucketName;

        Console.WriteLine("------------------------------------------------------------------");
        Console.WriteLine("Especifique o caminho para o arquivo/pasta que você deseja fazer backup.");
        Console.WriteLine("Exemplo: C:\\nomeDeUmaPasta\\NomeArquivo.txt");
        string? filePath = Console.ReadLine();

        if (string.IsNullOrWhiteSpace(filePath) || !Directory.Exists(filePath))
        {
            Console.WriteLine("❌ Caminho inválido ou inexistente.");
            return;
        }

        var transferUtility = new TransferUtility(s3Client);
        var uploadRequest = new TransferUtilityUploadDirectoryRequest
        {
            BucketName = bucketName,
            KeyPrefix = "Snorlax/",
            SearchOption = SearchOption.AllDirectories,
            Directory = filePath
        };

        Console.WriteLine("------------------------------------------------------------------");
        Console.WriteLine($"Deseja continuar o upload para o bucket: {bucketName}? (Y para continuar)");
        if (Console.ReadKey().Key == ConsoleKey.Y)
        {
            Console.WriteLine("\nIniciando upload...");
            try
            {
                await transferUtility.UploadDirectoryAsync(uploadRequest);
                Console.WriteLine("------------------------------------------------------------------");
                Console.WriteLine($"Bucket: {uploadRequest.BucketName}");
                Console.WriteLine($"Origem: {filePath}");
                Console.WriteLine("✅ Upload concluído com sucesso!");
            }
            catch (Exception ex)
            {
                Console.WriteLine("------------------------------------------------------------------");
                Console.WriteLine($"Erro durante o upload:\n{ex.Message}");
            }
        }
        else
        {
            Console.WriteLine("\nOperação cancelada pelo usuário.");
        }

        Console.WriteLine("Pressione Enter para sair...");
        Console.ReadLine();
    }
}
