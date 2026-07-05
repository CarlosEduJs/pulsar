import { ThemeLogo } from "../theme-logo";
import { Link } from "waku";

export function Footer() {
  return (
    <footer className="border-t border-border">
      <div className="mx-auto flex flex-col items-center justify-between gap-4 px-4 py-8 max-w-6xl border-l border-r border-border md:flex-row">
        <div className="flex items-center gap-2">
          <ThemeLogo />
        </div>

        <nav className="flex items-center gap-4 text-xs text-muted-foreground">
          <Link to="/docs/guide" className="hover:text-foreground transition-colors">
            Docs
          </Link>
          <Link
            to="https://github.com/CarlosEduJs/pulsar"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-foreground transition-colors"
          >
            GitHub
          </Link>
          <span className="text-muted-foreground/40">v0.5.0</span>
        </nav>
      </div>
    </footer>
  );
}
