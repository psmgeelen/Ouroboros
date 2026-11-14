# Terraspace project config.
# Stacks live in infra/app/stacks. Bootstrap is applied once from a local
# admin login. The relay stack is applied per environment by GitHub Actions.
Terraspace.configure do |config|
  config.logger.level = :info
end
