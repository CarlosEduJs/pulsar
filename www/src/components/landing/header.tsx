import { ThemeLogo } from '@/components/theme-logo';
import { Button } from '@/components/ui/button';
import { Link } from 'waku';

export function Header() {
  return (
    <header className="border-b border-border">
      <div className="mx-auto flex h-14 items-center justify-between px-4 max-w-6xl border-l border-r border-border">
        <Link to="/">
          <ThemeLogo />
        </Link>

        <nav className="flex items-center gap-1">
          <Link
            to="/docs"
          >
            <Button size="sm" variant="ghost">
              Docs
            </Button>
          </Link>
          <Link
            to="https://github.com/CarlosEduJs/pulsar"
            target="_blank"
            rel="noopener noreferrer"
          >
            <Button size="sm" variant="ghost">
              GitHub
            </Button>
          </Link>
          <Link to="/docs/getting-started">
            <Button size="sm">Get Started</Button>
          </Link>
        </nav>
      </div>
    </header>
  );
}
