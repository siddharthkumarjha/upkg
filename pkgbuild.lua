PkgInfo = {
	name = "starship",
	desc = "The cross-shell prompt for astronauts",
	ver = "1.23.0",
	rel = 1,
}

Url = "https://starship.rs/"
License = { "ISC" }
Depends = { "gcc-libs", "glibc" }
MakeDepends = { "cmake", "git", "rust" }
CheckDepends = { "python" }
OptDepends = { "ttf-font-nerd: Nerd Font Symbols preset" }

GitHubLink = "https://github.com" .. "/" .. PkgInfo.name .. "/" .. PkgInfo.name
Source = { "git+" .. GitHubLink .. ".git#tag=v" .. PkgInfo.ver }

Sha256Sums = { "SKIP" }

function Run(cmd)
	local handle = io.popen(cmd .. " 2>&1")
	local output = handle:read("*a")
	handle:close()
	return output
end

function GetHost(rustc_output)
	return string.match(rustc_output, "host:%s*(%S+)")
end

function Prepare()
	local rust_version = Run("rustc -vV")
	local target_host = GetHost(rust_version)

	local cargo_cmd = "cargo fetch --locked --target " .. target_host .. " --manifest-path starship/Cargo.toml"
	os.execute(cargo_cmd)
end

function Build()
	local env = 'CARGO_TARGET_DIR=target CFLAGS+=" -ffat-lto-objects"'
	local cargo_cmd = "cargo build --release --frozen --manifest-path starship/Cargo.toml"
	os.execute(env .. " " .. cargo_cmd)
end

function Check()
	local cargo_cmd = "cargo test --frozen --manifest-path starship/Cargo.toml"
	os.execute(cargo_cmd)
end

function Package()
	local install_starship = table.concat({
		"install -Dm 755 target/release/starship -t " .. PkgDir .. "/usr/bin",
		"install -Dm 644 starship/LICENSE -t " .. PkgDir .. "/usr/share/licenses/starship/",
		"install -dm 755 "
		.. PkgDir
		.. "/usr/share/{bash-completion/completions,elvish/lib,fish/vendor_completions.d,zsh/site-functions}/",
		"./target/release/starship completions bash > " .. PkgDir .. "/usr/share/bash-completion/completions/starship",
		"./target/release/starship completions elvish > " .. PkgDir .. "/usr/share/elvish/lib/starship.elv",
		"./target/release/starship completions fish > "
		.. PkgDir
		.. "/usr/share/fish/vendor_completions.d/starship.fish",
		"./target/release/starship completions zsh > " .. PkgDir .. "/usr/share/zsh/site-functions/_starship",
	}, "\n")
	os.execute(install_starship)
end

function ParseSourceUrl(source)
	local proto, rest = string.match(source, "^(.-)%+(.*)$")
	local url, tag = string.match(rest, "^(.-)#tag=(.*)$")
	return {
		proto = proto,
		full = source,
		url = url or rest,
		tag = tag,
	}
end

-- control flow
-- download source -> verify() -> extract source -> prepare() -> build() -> check() -> package()
function Test_fn()
	PkgDir = "/home/siddharth/tst/"
	for _, s in ipairs(Source) do
		local parsed_url = ParseSourceUrl(s)
		local clone_cmd = parsed_url.proto .. " clone " .. parsed_url.url .. " -b " .. parsed_url.tag
		os.execute(clone_cmd)
	end
	Prepare()
	Build()
	-- Check()
	Package()
end
