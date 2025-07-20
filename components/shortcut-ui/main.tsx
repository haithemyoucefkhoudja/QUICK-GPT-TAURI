// MainComponent.tsx
import { ChatInput } from "./input";
import WebSearchIndicator from "../WebSearchIndicator";
import HideButton from "../HideButton";
import { memo, useEffect, useMemo, useState } from "react";
import { CommandControl } from "@/src/command-list";
import ChatIndicator from "../ChatIndicator";
import { useAppResize } from "./hooks/use-app-resize";
import ExpandIndicator from "../ExpandIndicator";
import MessageList from "./message-list";
import ConfigManager from "./config-manager";
// Message type is not directly used in MainComponent props, but likely by useChat
// import { Message } from "@/types/Message";
import SettingsIndicator from "../SettingsIndicator";
import ThemeIndicator from "../themeIndicator";
import HistoryIndicator from "../HistoryIndicator";
import { ConversationList } from "./history";
import { useChat } from "@/providers/chat-provider";
import DonationIndicator from "./donation-indicator";
import NewChatIndicator from "./new-chat-indicator";
import { cn } from "@/lib/utils";
import { ErrorMessage } from "./error-message";
import DonationForm from "./donation-form";
import { useConfig } from "@/providers/config-provider";
import { Button } from "../ui/button";
import { ArrowLeft, ArrowRight } from "lucide-react";

type WindowKey =
  | "chat"
  | "commands"
  | "list"
  | "settings"
  | "history"
  | "donation";
const TitleComponent = ({ title }: { title: string }) => {
  const [showMore, setShowMore] = useState(false);
  const { config } = useConfig();
  const { LAYOUT_MODE } = config;
  const truncateText = (text: string, maxLength: number) => {
    if (text.length <= maxLength) return text;
    return text.substring(0, maxLength) + "...";
  };

  return (
    <div
      className="flex justify-between w-full h-fit   max-w-full relative"
      data-tauri-drag-region
    >
      <h3
        onMouseEnter={() => setShowMore(true)}
        onMouseLeave={() => setShowMore(false)}
        className="h-fit w-full text-md font-bold text-primary truncate break-words my-2 mx-4 max-w-full"
        data-tauri-drag-region
      >
        {truncateText(title, 30)}
      </h3>
      {showMore && (
        <p className="absolute top-full left-4 z-50 bg-popover text-popover-foreground px-2 py-1 rounded text-xs border shadow-md text-wrap">
          {truncateText(title, 100)}
        </p>
      )}
      <NewChatIndicator />
    </div>
  );
};
const MainWrapper = memo(function MainWrapper({
  children,
}: {
  children: React.ReactNode;
}) {
  const { mainRef, ActiveWindow } = useAppResize();
  const { config } = useConfig();
  const { lastMessage } = useChat();
  const lastMessageVisible = !!lastMessage && !lastMessage.hidden;
  const { LAYOUT_MODE } = config;

  return (
    <div
      ref={mainRef}
      className={cn(
        "",
        LAYOUT_MODE === "vertical" ? "" : "min-h-64 max-h-full w-full"
      )}
      data-tauri-drag-region
    >
      <div
        className={cn(
          "w-full h-full",
          LAYOUT_MODE === "vertical" ? "" : "flex flex-col"
        )}
        data-tauri-drag-region
      >
        <div
          className={cn(
            "flex bg-background relative w-full p-4 rounded-xl border-2 border-foreground/20",

            ActiveWindow === "chat" ? "flex-1 " : ""
          )}
          // style={widthFunc(ActiveWindow, LAYOUT_MODE, !!lastMessage)}
          data-tauri-drag-region
        >
          {children}
        </div>
        {ActiveWindow === "chat" && lastMessageVisible && (
          <div
            className={cn("h-full", LAYOUT_MODE === "vertical" ? "w-full" : "")}
            data-tauri-drag-region
          ></div>
        )}
      </div>
    </div>
  );
});
const MainComponent = memo(function MainComponent() {
  const { ActiveWindow, setActiveWindow, mainRef } = useAppResize();
  const { config } = useConfig();
  const { LAYOUT_MODE } = config;
  const { setError, conversation, isLoading, error, closeLastMessage } =
    useChat(); // Destructure all props needed by child components here
  const closeWindowOnChatInactive = () => {
    // toggleLastMessage();
    setError(null);
    closeLastMessage();
  };

  useEffect(() => {
    if (ActiveWindow !== "chat") {
      closeWindowOnChatInactive();
    }
  }, [ActiveWindow]); // Removed closeWindow from deps as it's stable

  // Define all possible window components and their props
  // Memoize this structure if props within it are stable, or ensure props from useChat are stable
  const windowDefinitions = useMemo(() => {
    return [
      {
        key: "chat" as WindowKey,
        // Pass necessary props. Ensure these props are stable (memoized if objects/arrays/functions)
        // to prevent re-renders of ChatInput when it's not the active one changing.
        Component: () => <ChatInput />,
      },
      {
        key: "commands" as WindowKey,
        Component: () => <CommandControl />,
      },
      {
        key: "list" as WindowKey,
        // Example: if MessageList needs scrollToMessageId
        // Component: () => <MessageList scrollToMessageId={someIdFromState} />,
        Component: () => <MessageList />,
      },
      {
        key: "settings" as WindowKey,
        Component: () => <ConfigManager />,
      },
      {
        key: "history" as WindowKey,
        Component: () => <ConversationList />,
      },
      {
        key: "donation" as WindowKey,
        Component: () => <DonationForm />,
      },
    ];
  }, [isLoading, conversation]);

  return (
    <MainWrapper>
      {/* Left Sidebar */}
      <div
        className="flex flex-col items-center gap-2 h-full justify-end mt-auto shrink-0"
        data-tauri-drag-region
      >
        <WebSearchIndicator />
        <HistoryIndicator />
        <ChatIndicator />
        <ExpandIndicator />
      </div>

      {/* Main Content Area - This will contain the sliding windows */}
      <div
        className="w-full flex-1 flex flex-col overflow-hidden relative mx-2"
        data-tauri-drag-region
      >
        {" "}
        {/* Added relative and overflow-hidden */}
        <TitleComponent title={conversation?.title || "New Chat"} />
        {error && <ErrorMessage message={error} type="list" />}
        {/* Container for the sliding windows */}
        <div className="flex-1 relative h-full">
          {" "}
          {/* Ensures this container takes up remaining space and is a positioning context */}
          <div
            className={cn(
              "w-full ",
              LAYOUT_MODE === "vertical" ? "h-full" : ""
            )}
          >
            {windowDefinitions.find((w) => w.key === ActiveWindow)?.Component()}
          </div>
        </div>
      </div>

      {/* Right Sidebar */}
      <div
        className="flex flex-col items-center gap-2 justify-between shrink-0"
        data-tauri-drag-region
      >
        <HideButton />
        <div
          className="flex items-center flex-col gap-2"
          data-tauri-drag-region
        >
          <ThemeIndicator />
          <SettingsIndicator
            activeWindow={ActiveWindow}
            setActiveWindow={() => setActiveWindow("settings")}
          />
          <DonationIndicator
            activeWindow={ActiveWindow}
            setActiveWindow={() => setActiveWindow("donation")}
          />

          <HiddenIndicator />
        </div>
      </div>
    </MainWrapper>
  );
});
const HiddenIndicator = () => {
  const { toggleLastMessage, lastMessage } = useChat();
  const hidden = lastMessage?.hidden;
  const [isRotating, setIsRotating] = useState(false);
  useEffect(() => {
    if (isRotating) {
      const timer = setTimeout(() => setIsRotating(false), 300);
      return () => clearTimeout(timer);
    }
  }, [isRotating]);
  const handleClick = () => {
    setIsRotating(true);
    toggleLastMessage();
  };
  const getIcon = () => {
    if (hidden) {
      return <ArrowRight className="h-[1.2rem] w-[1.2rem]" />;
    } else {
      return <ArrowLeft className="h-[1.2rem] w-[1.2rem]" />;
    }
  };
  return (
    <Button
      onClick={handleClick}
      variant="outline"
      size="icon"
      className={`rounded-full transition-transform duration-300 ease-in-out ${
        isRotating ? "rotate-180" : ""
      }`}
    >
      <div
        className={`transition-opacity duration-300 ease-in-out ${
          isRotating ? "opacity-0" : "opacity-100"
        }`}
      >
        {getIcon()}
      </div>
    </Button>
  );
};
MainComponent.displayName = "MainComponent";
export default MainComponent;
