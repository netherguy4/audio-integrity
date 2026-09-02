using System.Net.Http.Headers;
using System.Text;
using System.Text.Json;
using NLog;
using NzbDrone.Core.DecisionEngine;
using NzbDrone.Core.Download;
using NzbDrone.Core.MediaFiles.TrackImport;
using NzbDrone.Core.Parser.Model;

namespace NzbDrone.Core.Plugins
{
    public sealed class AudioIntegrityPlugin : Plugin
    {
        public override string Name => "Audio Integrity";
        public override string Owner => "netherguy4";
        public override string GithubUrl => "https://github.com/netherguy4/audio-integrity";
    }
}

namespace NzbDrone.Core.MediaFiles.TrackImport.Specifications
{
    public sealed class AudioIntegritySpecification : IImportDecisionEngineSpecification<LocalTrack>
    {
        private static readonly HashSet<string> SupportedExtensions = new(StringComparer.OrdinalIgnoreCase)
        {
            ".flac", ".mp3", ".wav", ".m4a", ".ape", ".opus"
        };

        private readonly Logger _logger;

        public AudioIntegritySpecification(Logger logger)
        {
            _logger = logger;
        }

        public Decision IsSatisfiedBy(LocalTrack item, DownloadClientItem downloadClientItem)
        {
            if (item.ExistingFile || !SupportedExtensions.Contains(Path.GetExtension(item.Path)))
            {
                return Decision.Accept();
            }

            var config = LoadConfig();
            var serviceUrl = (Environment.GetEnvironmentVariable("AUDIO_INTEGRITY_URL") ?? config?.Url ?? "http://audio-integrity:8080").TrimEnd('/');
            var token = Environment.GetEnvironmentVariable("AUDIO_INTEGRITY_TOKEN") ?? config?.Token;
            if (string.IsNullOrWhiteSpace(serviceUrl) || string.IsNullOrWhiteSpace(token))
            {
                return Decision.Reject("Audio Integrity is not configured; refusing import");
            }

            try
            {
                using var client = new HttpClient { Timeout = TimeSpan.FromMinutes(15) };
                client.DefaultRequestHeaders.Add("X-Integrity-Token", token);
                client.DefaultRequestHeaders.Accept.Add(new MediaTypeWithQualityHeaderValue("application/json"));
                var request = JsonSerializer.Serialize(new VerifyRequest(item.Path));
                using var response = client.PostAsync(
                    $"{serviceUrl}/internal/lidarr/verify",
                    new StringContent(request, Encoding.UTF8, "application/json")
                ).GetAwaiter().GetResult();
                var payload = response.Content.ReadAsStringAsync().GetAwaiter().GetResult();
                var verdict = JsonSerializer.Deserialize<VerifyResponse>(payload, new JsonSerializerOptions
                {
                    PropertyNameCaseInsensitive = true
                });

                if (response.IsSuccessStatusCode && verdict?.Accepted == true)
                {
                    if (string.Equals(verdict.Authenticity, "likely_lossy", StringComparison.Ordinal))
                    {
                        _logger.Warn("Audio Integrity accepted structurally healthy file with lossy-source evidence: {0}: {1}", item.Path, verdict.AuthenticityMessage);
                    }
                    return Decision.Accept();
                }

                var reason = verdict?.Message ?? $"service returned HTTP {(int)response.StatusCode}";
                return Decision.Reject($"Audio Integrity rejected this file: {reason}");
            }
            catch (Exception exception)
            {
                _logger.Error(exception, "Audio Integrity verification failed for {0}", item.Path);
                return Decision.Reject($"Audio Integrity could not verify this file: {exception.Message}");
            }
        }

        private static IntegrityConfig? LoadConfig()
        {
            var path = Environment.GetEnvironmentVariable("AUDIO_INTEGRITY_CONFIG") ?? "/config/audio-integrity.json";
            if (!File.Exists(path))
            {
                return null;
            }
            return JsonSerializer.Deserialize<IntegrityConfig>(File.ReadAllText(path), new JsonSerializerOptions
            {
                PropertyNameCaseInsensitive = true
            });
        }

        private sealed record VerifyRequest(string Path);
        private sealed record IntegrityConfig(string Url, string Token);

        private sealed record VerifyResponse(
            bool Accepted,
            string Verdict,
            bool Cached,
            string Validator,
            string Authenticity,
            string AuthenticityMessage,
            string Message
        );
    }
}
