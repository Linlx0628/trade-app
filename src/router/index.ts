import { createRouter, createWebHashHistory } from "vue-router"

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      name: "dashboard",
      component: () => import("@/views/dashboard/DashboardView.vue"),
      meta: { title: "仪表盘" },
    },
    {
      path: "/trade-plan",
      name: "trade-plan",
      component: () => import("@/views/trade-plan/TradePlanView.vue"),
      meta: { title: "交易计划" },
    },
    {
      path: "/trade-log",
      name: "trade-log",
      component: () => import("@/views/trade-log/TradeLogView.vue"),
      meta: { title: "交易日志" },
    },
    {
      path: "/trade-summary",
      name: "trade-summary",
      component: () => import("@/views/trade-summary/TradeSummaryView.vue"),
      meta: { title: "交易总结" },
    },
    {
      path: "/settings",
      name: "settings",
      component: () => import("@/views/settings/SettingsView.vue"),
      meta: { title: "账户设置" },
    },
    {
      path: "/signal-monitor",
      name: "signal-monitor",
      component: () => import("@/views/signal-monitor/SignalMonitorView.vue"),
      meta: { title: "行情监控" },
    },
  ],
})

export default router
