require 'rake'

Gem::Specification.new do |spec|
  spec.name          = "minify_html"
  spec.version       = "0.8.4"
  spec.authors       = ["Wilson Lin"]
  spec.email         = ["code@wilsonl.in"]
  spec.license       = "MIT"
  spec.files         = FileList["lib/*", "README.md"].to_a
  spec.summary       = "Extremely fast and smart HTML + JS + CSS minifier"
  spec.homepage      = "https://github.com/wilsonzlin/minify-html"

  spec.require_paths = ["lib"]

  spec.add_dependency "fiddle", "~> 1.0"
end
