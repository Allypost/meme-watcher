import Link from "next/link";

export default function DefaultLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex min-h-screen flex-col bg-gradient-to-br from-[#2e026d] to-[#15162c]">
      <header className="container mt-12 flex h-12 flex-col">
        <Link
          className="h-full-w-auto mx-auto text-4xl font-bold tracking-wider"
          href="/"
        >
          Meme Watcher
        </Link>
      </header>

      <main className="flex flex-col gap-4 px-4 pt-12 sm:px-8">{children}</main>

      <footer className="mt-auto">
        <div className="container mt-12 rounded-t-lg bg-black p-4 text-gray-300">
          Copyright &copy; {new Date().getFullYear()}{" "}
          <a
            href="https://github.com/allypost"
            rel="noopener noreferrer"
            target="_blank"
          >
            Allypost
          </a>
        </div>
      </footer>
    </div>
  );
}
