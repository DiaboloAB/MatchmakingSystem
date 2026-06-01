import { LivePlayerChart } from "@/components/live-player-chart"
import { RankDistributionChart } from "@/components/rank-distribution-chart"
import { RecentGameCard } from "@/components/recent-game-card"
import { SectionCards } from "@/components/section-cards"
import { SiteHeader } from "@/components/site-header"
import { TopPlayersCard } from "@/components/top-player-card"


export default function Page() {
  return (
    <div>

      <SiteHeader name="Dashboard" />
      <div className="flex flex-1 flex-col">
        <div className="@container/main flex flex-1 flex-col gap-2">
          <div className="flex flex-col gap-4 py-4 md:gap-6 md:py-6">
            <SectionCards />

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 md:gap-6 px-4">
              <LivePlayerChart />

              <RankDistributionChart />
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 md:gap-6 px-4">
              <div className="lg:col-span-1">
                <TopPlayersCard />
              </div>

              <div className="lg:col-span-2">
                <RecentGameCard />
              </div>
            </div>

          </div>
        </div>
      </div>
    </div>
  )
}
