"use client";

import { useNotebookLayout } from "fumadocs-ui/layouts/notebook";
import { isLayoutTabActive, LinkItem, type LayoutTab } from "fumadocs-ui/layouts/shared";
import { usePathname } from "fumadocs-core/framework";
import Link from "fumadocs-core/link";
import { buttonVariants } from "fumadocs-ui/components/ui/button";
import { cn } from "cnfast";
import { ChevronDown, Languages, Sidebar } from "lucide-react";
import { useMemo, useRef, useState, type ComponentProps } from "react";

export function DocsHeader(props: ComponentProps<"header">) {
  const {
    slots,
    navItems,
    isNavTransparent,
    props: { tabMode, nav, tabs, sidebar },
  } = useNotebookLayout();
  const { open } = slots.sidebar?.useSidebar?.() ?? {};
  const navMode = nav?.mode ?? "auto";
  const sidebarCollapsible = sidebar.collapsible ?? true;
  const showLayoutTabs = tabMode === "navbar" && tabs.length > 0;

  return (
    <header
      id="nd-subnav"
      data-transparent={isNavTransparent && !open}
      {...props}
      className={cn(
        "sticky [grid-area:header] flex flex-col top-(--fd-docs-row-1) z-10 backdrop-blur-sm transition-colors data-[transparent=false]:bg-fd-background/80 layout:[--fd-header-height:--spacing(14)]",
        showLayoutTabs && "lg:layout:[--fd-header-height:--spacing(24)]",
        props.className,
      )}
    >
      <div data-header-body="" className="flex border-b px-4 gap-2 h-14 md:px-6">
        <div
          className={cn(
            "items-center",
            navMode === "top" && "flex flex-1",
            navMode === "auto" && "hidden has-data-[collapsed=true]:md:flex max-md:flex",
          )}
        >
          {sidebarCollapsible && slots.sidebar && navMode === "auto" && (
            <slots.sidebar.collapseTrigger
              className={cn(
                buttonVariants({ color: "ghost", size: "icon-sm" }),
                "-ms-1.5 text-fd-muted-foreground data-[collapsed=false]:hidden max-md:hidden",
              )}
            >
              <Sidebar />
            </slots.sidebar.collapseTrigger>
          )}
          {slots.navTitle && (
            <slots.navTitle
              className={cn(
                "inline-flex items-center gap-2.5 font-semibold",
                navMode === "auto" && "md:hidden",
              )}
            />
          )}
          {nav?.children}
        </div>

        {slots.searchTrigger && (
          <slots.searchTrigger.full
            hideIfDisabled
            className={cn(
              "w-full my-auto max-md:hidden",
              navMode === "top" ? "ps-2.5 rounded-xl max-w-sm" : "max-w-[240px]",
            )}
          />
        )}

        <div className="flex flex-1 items-center justify-end md:gap-2">
          <div className="flex items-center gap-6 empty:hidden max-lg:hidden">
            {navItems
              .filter((item) => item.type !== "icon")
              .map((item, i) => (
                <NavbarLinkItem item={item} key={i} />
              ))}
          </div>

          {navItems
            .filter((item) => item.type === "icon")
            .map((item, i) => (
              <LinkItem
                item={item}
                key={i}
                className={cn(
                  buttonVariants({ size: "icon-sm", color: "ghost" }),
                  "text-fd-muted-foreground max-lg:hidden",
                )}
                aria-label={item.label}
              >
                {item.icon}
              </LinkItem>
            ))}

          <div className="flex items-center md:hidden">
            {slots.searchTrigger && <slots.searchTrigger.sm hideIfDisabled className="p-2" />}
            {slots.sidebar && (
              <slots.sidebar.trigger
                className={cn(
                  buttonVariants({
                    color: "ghost",
                    size: "icon-sm",
                    className: "p-2 -me-1.5",
                  }),
                )}
              >
                <Sidebar />
              </slots.sidebar.trigger>
            )}
          </div>

          <div className="flex items-center gap-2 max-md:hidden">
            {slots.languageSelect && (
              <slots.languageSelect.root>
                <Languages className="size-4.5 text-fd-muted-foreground" />
              </slots.languageSelect.root>
            )}
            {slots.themeSwitch && <slots.themeSwitch />}
            {sidebarCollapsible && slots.sidebar && navMode === "top" && (
              <slots.sidebar.collapseTrigger
                className={cn(
                  buttonVariants({ color: "secondary", size: "icon-sm" }),
                  "text-fd-muted-foreground rounded-full -me-1.5",
                )}
              >
                <Sidebar />
              </slots.sidebar.collapseTrigger>
            )}
          </div>
        </div>
      </div>

      {showLayoutTabs && <LayoutTabs tabs={tabs} />}
    </header>
  );
}

function LayoutTabs({ tabs, className, ...props }: { tabs: LayoutTab[]; className?: string }) {
  const pathname = usePathname();
  const selectedIdx = useMemo(() => {
    let last = -1;
    tabs.forEach((option, i) => {
      if (isLayoutTabActive(option, pathname)) last = i;
    });
    return last;
  }, [tabs, pathname]);

  return (
    <div
      className={cn(
        "flex flex-row items-end gap-6 overflow-x-auto border-b px-6 h-10 max-lg:hidden",
        className,
      )}
      {...props}
    >
      {tabs.map((option, i) => {
        const { title, url, unlisted, icon, props: { className: cls, ...rest } = {} } = option;
        const isSelected = selectedIdx === i;

        return (
          <Link
            key={i}
            href={url}
            className={cn(
              "inline-flex border-b-2 border-transparent transition-colors items-center pb-1.5 font-medium gap-1.5 text-fd-muted-foreground text-sm text-nowrap hover:text-fd-accent-foreground",
              unlisted && !isSelected && "hidden",
              isSelected && "border-fd-primary text-fd-primary",
              cls,
            )}
            {...rest}
          >
            {icon && <span className="size-4 [&_svg]:size-full">{icon}</span>}
            {title}
          </Link>
        );
      })}
    </div>
  );
}

function NavbarLinkItem({ item, className, ...props }: { item: any; className?: string }) {
  if (item.type === "custom") return item.children;
  if (item.type === "menu")
    return <NavbarLinkItemMenu item={item} className={className} {...props} />;

  return (
    <LinkItem
      item={item}
      className={cn(
        "text-sm text-fd-muted-foreground transition-colors hover:text-fd-accent-foreground data-[active=true]:text-fd-primary",
        className,
      )}
      {...props}
    >
      {item.text}
    </LinkItem>
  );
}

function NavbarLinkItemMenu({
  item,
  hoverDelay = 50,
  className,
  ...props
}: {
  item: any;
  hoverDelay?: number;
  className?: string;
}) {
  const [open, setOpen] = useState(false);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const freezeUntil = useRef<number | null>(null);

  const delaySetOpen = (value: boolean) => {
    if (timeoutRef.current) clearTimeout(timeoutRef.current);
    timeoutRef.current = setTimeout(() => {
      setOpen(value);
      freezeUntil.current = Date.now() + 300;
    }, hoverDelay);
  };

  const onPointerEnter = (e: React.PointerEvent) => {
    if (e.pointerType === "touch") return;
    delaySetOpen(true);
  };

  const onPointerLeave = (e: React.PointerEvent) => {
    if (e.pointerType === "touch") return;
    delaySetOpen(false);
  };

  return (
    <div
      className={cn("relative inline-flex", className)}
      {...props}
      onPointerEnter={onPointerEnter}
      onPointerLeave={onPointerLeave}
    >
      <button
        className="inline-flex items-center gap-1.5 p-1 text-sm text-fd-muted-foreground transition-colors has-data-[active=true]:text-fd-primary data-[state=open]:text-fd-accent-foreground focus-visible:outline-none"
        onClick={() => setOpen(!open)}
      >
        {item.url ? <LinkItem item={item}>{item.text}</LinkItem> : item.text}
        <ChevronDown className="size-3" />
      </button>
      {open && (
        <div className="absolute top-full left-0 flex flex-col p-1 text-fd-muted-foreground text-start bg-fd-popover border rounded-lg shadow-lg min-w-[180px] mt-1 z-50">
          {item.items.map((child: any, i: number) => {
            if (child.type === "custom") return <>{child.children}</>;
            return (
              <LinkItem
                key={i}
                item={child}
                className="inline-flex items-center gap-2 rounded-md p-2 transition-colors hover:bg-fd-accent hover:text-fd-accent-foreground data-[active=true]:text-fd-primary [&_svg]:size-4"
                onClick={() => setOpen(false)}
              >
                {child.icon}
                {child.text}
              </LinkItem>
            );
          })}
        </div>
      )}
    </div>
  );
}
