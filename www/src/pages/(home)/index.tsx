import { Hero } from "@/components/landing/hero";
import { Features } from "@/components/landing/features";
import { Demo } from "@/components/landing/demo";
import { Footer } from "@/components/landing/footer";

export default function Home() {
  return (
    <>
      <title>Pulsar — Static Analyzer for TypeScript ORM Code</title>
      <meta
        name="description"
        content="Detect quality, performance, and schema consistency issues in your TypeScript ORM code before they reach production."
      />
      <meta property="og:image" content="/banner.png" />
      <meta
        property="og:title"
        content="Pulsar — Static Analyzer for TypeScript ORM Code"
      />
      <meta
        property="og:description"
        content="Detect quality, performance, and schema consistency issues in your TypeScript ORM code before they reach production."
      />
      <main className="flex-1 flex flex-col">
        <div className="mx-auto w-full max-w-6xl border-l border-r border-border">
          <section className="border-b border-border">
            <Hero />
          </section>
          <section className="border-b border-border">
            <Features />
          </section>
          <section>
            <Demo />
          </section>
        </div>
        <Footer />
      </main>
    </>
  );
}

export async function getConfig() {
  return {
    render: "static",
  };
}
