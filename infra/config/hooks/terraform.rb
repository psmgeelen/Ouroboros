require "open3"

# Runs before Terraform init, plan, and apply.
# Refreshes GOOGLE_OAUTH_ACCESS_TOKEN from the current gcloud user so a stale
# application-default credential cannot fail the run with invalid_rapt.
# Bootstrap also fills TF_VAR_devops_project_number when it is unset.
class GcloudUserToken
  def call(runner)
    return if ENV["GITHUB_ACTIONS"] == "true"

    script = File.expand_path("../../../scripts/gcp-user-auth.sh", __dir__)
    with_number = runner.mod.name == "bootstrap" ? "1" : "0"
    output, status = Open3.capture2(
      { "GCP_AUTH_WITH_PROJECT_NUMBER" => with_number },
      "bash", script, "print"
    )
    raise "gcloud user auth failed" unless status.success?

    token, number = output.lines.map(&:strip)
    raise "gcloud auth print-access-token returned an empty token" if token.to_s.empty?

    ENV["GOOGLE_OAUTH_ACCESS_TOKEN"] = token
    if with_number == "1" && ENV["TF_VAR_devops_project_number"].to_s.empty?
      raise "could not read the ouroboros-devops project number" if number.to_s.empty?

      ENV["TF_VAR_devops_project_number"] = number
    end
  end
end

before("init", "plan", "apply",
  label: "gcloud user token",
  execute: GcloudUserToken
)
