import { Sidebar } from "./components/Sidebar";
import { MobileNav } from "./components/MobileNav";
import { Hero } from "./components/Hero";
import { Overview } from "./components/Overview";
import { HowItWorks } from "./components/HowItWorks";
import { Install } from "./components/Install";
import { Commands } from "./components/Commands";
import { Privacy } from "./components/Privacy";
import { Requirements } from "./components/Requirements";
import { Troubleshooting } from "./components/Troubleshooting";
import { Footer } from "./components/Footer";

export default function Page() {
  return (
    <>
      <MobileNav />
      <div className="lg:flex">
        <Sidebar />
        <div className="min-w-0 flex-1">
          <main
            id="top"
            className="mx-auto max-w-[720px] px-6 pb-28 pt-12 md:px-8 md:pt-16 lg:px-14 lg:pt-20 xl:px-20"
          >
            <Hero />
            <Overview />
            <HowItWorks />
            <Install />
            <Commands />
            <Privacy />
            <Requirements />
            <Troubleshooting />
          </main>
          <Footer />
        </div>
      </div>
    </>
  );
}
