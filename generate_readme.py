import re
from pathlib import Path

text = Path("src/lib.rs").read_text(encoding="utf-8")
Path("README.md").write_text("# softbuffer-rgb\n\n" + "\n".join(re.findall(r"^//!(.*)$", text, flags=re.MULTILINE)))