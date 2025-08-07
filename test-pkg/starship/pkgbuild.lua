Proto = {
	git = 1,
	patch = 2,
}

-- Create reverse lookup
ProtoStr = {}
for k, v in pairs(Proto) do
	ProtoStr[v] = k
end

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
Source = {
	{
		proto = Proto.git,
		url = GitHubLink,
		tag = "v" .. PkgInfo.ver,
		directory = "starship",
	},
	{ proto = Proto.patch, file = "./0001-fix-rust-1.89.0-warnings-and-errors-blocking-CI-pipe.patch" },
	{ proto = Proto.patch, file = "./0002-fix-git-tests-spawning-an-editor.patch" },
}

Sha256Sums = {
	"SKIP",
	"2e66eff0249f87f1deb1dfd0916d1017c1772a05a7627668d8855a3f227908e8",
	"41e085267c1a8c60b29442a8376c4cf2c1f98f658b13ff17370887413047e7f4",
}

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
	for _, s in ipairs(Source) do
		if s.proto == Proto.patch then
			local patch_cmd = ProtoStr[s.proto] .. " -d starship -p1 < " .. s.file
			print(patch_cmd)
			os.execute(patch_cmd)
		end
	end

	local rust_version = Run("rustc -vV")
	local target_host = GetHost(rust_version)

	local cargo_cmd = "cargo fetch --locked --target " .. target_host .. " --manifest-path starship/Cargo.toml"
	print(cargo_cmd)
	os.execute(cargo_cmd)
end

function Build()
	local env = 'CARGO_TARGET_DIR=target CFLAGS+=" -ffat-lto-objects"'
	local cargo_cmd = "cargo build --release --frozen --manifest-path starship/Cargo.toml"
	print(cargo_cmd)
	os.execute(env .. " " .. cargo_cmd)
end

function Check()
	local cargo_cmd = "cargo test --frozen --manifest-path starship/Cargo.toml"
	print(cargo_cmd)
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
	print(install_starship)
	os.execute(install_starship)
end

local function get_filename_from_url(url)
	-- Strip query string and fragment
	local cleaned = url:match("^[^?#]+")
	-- Extract last part after final slash
	return cleaned:match("^.+/(.+)$")
end

function Verify()
	for i, s in ipairs(Source) do
		local file_name
		if s.file ~= nil then
			file_name = s.file
		elseif s.url ~= nil then
			file_name = get_filename_from_url(s.url)
		else
			error("no file passed to source?")
		end

		if Sha256Sums[i] ~= "SKIP" then
			local actual_sha = Run("sha256sum " .. file_name):match("^([a-f0-9]+)")
			if actual_sha ~= Sha256Sums[i] then
				error(
					"Sha256Sums mismatch, expected: "
					.. Sha256Sums[i]
					.. " got "
					.. actual_sha
					.. " for file "
					.. file_name
				)
			else
				print("sha256 for " .. file_name .. " successfully validated")
			end
		else
			print("for ", file_name, " SKIP sha check requested")
		end
	end
end

function Fetch()
	for _, s in ipairs(Source) do
		if s.proto == Proto.git then
			local clone_cmd = ProtoStr[s.proto] .. " clone " .. s.url .. " -b " .. s.tag .. " " .. s.directory
			print(clone_cmd)
			os.execute(clone_cmd)
		end
	end
end

-- control flow
-- download source -> verify() -> extract source -> prepare() -> build() -> check() -> package()
function Test_fn()
	PkgDir = "/home/siddharth/tst/"
	Fetch()
	Verify()
	Prepare()
	Build()
	Check()
	Package()
end
