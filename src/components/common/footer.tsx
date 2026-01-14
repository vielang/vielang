"use client";

import { Facebook, Github, Youtube } from "lucide-react";
import Link from "next/link";

import { cn } from "../../lib/utils";
import { Button } from "../custom/button";
import { useTranslation } from "react-i18next";

export function Footer({ className }: { className?: string }) {
  const { t } = useTranslation();
  return (
    <footer className={cn("border-t bg-background mt-12", className)}>
      <div className="max-w-6xl mx-auto px-4 py-8">
        <div className="grid grid-cols-1 gap-8 md:grid-cols-4">
          {/* Brand Section */}
          <div className="space-y-4">
            <Link href="/" className="flex items-center gap-2">
              <span className="text-lg font-semibold text-foreground">
                VieLang
              </span>
            </Link>
            <p className="text-sm text-muted-foreground">
              {t("footer.description")}
            </p>
            <div className="flex space-x-2">
              <a href="https://www.facebook.com/khieu.dv96" target="_blank" rel="noopener noreferrer">
                <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
                  <Facebook className="h-4 w-4" />
                  <span className="sr-only">Facebook</span>
                </Button>
              </a>
              <a href="https://www.youtube.com/@vie-vlogs" target="_blank" rel="noopener noreferrer">
                <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
                  <Youtube className="h-4 w-4" />
                  <span className="sr-only">YouTube</span>
                </Button>
              </a>
              <a href="https://github.com/khieu-dv" target="_blank" rel="noopener noreferrer">
                <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
                  <Github className="h-4 w-4" />
                  <span className="sr-only">GitHub</span>
                </Button>
              </a>
            </div>
          </div>

          {/* Posts Section */}
          <div>
            <h3 className="mb-3 text-sm font-medium text-foreground">{t("footer.posts")}</h3>
            <ul className="space-y-2 text-sm">
              <li>
                <Link
                  href="/docs"
                  className="text-muted-foreground hover:text-foreground transition-colors"
                >
                  {t("footer.allPosts")}
                </Link>
              </li>
              <li>
                <Link
                  href="/docs"
                  className="text-muted-foreground hover:text-foreground transition-colors"
                >
                  {t("footer.goCourse")}
                </Link>
              </li>
              <li>
                <Link
                  href="/docs"
                  className="text-muted-foreground hover:text-foreground transition-colors"
                >
                  {t("footer.ginCourse")}
                </Link>
              </li>
            </ul>
          </div>

          {/* Community Section */}
          <div>
            <h3 className="mb-3 text-sm font-medium text-foreground">{t("footer.community")}</h3>
            <ul className="space-y-2 text-sm">
              <li>
                <Link
                  href="/chat"
                  className="text-muted-foreground hover:text-foreground transition-colors"
                >
                  {t("footer.chatWithUs")}
                </Link>
              </li>
              <li>
                <Link
                  href="/docs"
                  className="text-muted-foreground hover:text-foreground transition-colors"
                >
                  {t("footer.discussions")}
                </Link>
              </li>
            </ul>
          </div>
        </div>

        {/* Bottom Section */}
        <div className="mt-8 pt-6 border-t">
          <div className="flex flex-col items-center justify-between gap-4 md:flex-row">
            <p className="text-sm text-muted-foreground">
              &copy; {new Date().getFullYear()} VieLang. {t("footer.allRightsReserved")}
            </p>
            <div className="flex items-center gap-4 text-sm">
              <Link href="/privacy" className="text-muted-foreground hover:text-foreground transition-colors">
                {t("footer.privacy")}
              </Link>
              <Link href="/terms" className="text-muted-foreground hover:text-foreground transition-colors">
                {t("footer.terms")}
              </Link>
              <Link href="/contact" className="text-muted-foreground hover:text-foreground transition-colors">
                {t("footer.contact")}
              </Link>
            </div>
          </div>
        </div>
      </div>
    </footer>
  );
}
