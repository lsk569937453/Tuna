import { Button, Card, Pagination } from "flowbite-react";
import { Table } from "flowbite-react";
import { HiOutlinePlus, HiSearch, HiShoppingCart } from "react-icons/hi";
import ReactECharts from 'echarts-for-react';
import Request from "./utils/axiosUtils";
import { useEffect, useState } from "react"; import BatteryGauge from 'react-battery-gauge'
import moment from 'moment';


const pageSize = 5;

function Dashboard() {

    const [dataPerMinute, setDataPerMinute] = useState([]);

    const [datePerDay, setDatePerDay] = useState([]);
    const [dataPerMinuteGroupByTaskId, setDataPerMinuteGroupByTaskId] = useState([]);

    useEffect(() => {
        getDataPerMinute();
        getDataPerDay();
        getDataPerMinuteGroupByTaskId();
    }, []);

    const getDataPerMinute = () => {
        Request.get("/api/sqlLogs/perMinute").then((res) => {
            console.log(res);
            const mesArray = res.data.message.map(
                ({
                    minute: minute,
                    total_logs: total_logs,
                }) => {
                    return {
                        minute,
                        total_logs,
                    };
                }
            );
            setDataPerMinute(mesArray);
        });
    };
    const getDataPerDay = () => {
        Request.get("/api/sqlLogs/perDay").then((res) => {
            console.log(res);
            const mesArray = res.data.message.map(
                ({
                    day: day,
                    total_logs: total_logs,
                }) => {
                    return {
                        day,
                        total_logs,
                    };
                }
            );
            setDatePerDay(mesArray);
        });
    };
    const getDataPerMinuteGroupByTaskId = () => {
        Request.get("/api/sqlLogs/perMinuteTaskId").then((res) => {
            console.log(res);

            setDataPerMinuteGroupByTaskId(res.data.message);
        });
    };
    const optionsForDataPerMinute = () => {


        return {
            title: {
                // left: 'center',
                text: '今日同步总览'
            },
            xAxis: {
                type: 'category',
                data: dataPerMinute.map(item => item.minute)
            },
            yAxis: {
                name: '同步次数',

                type: 'value',

            },
            series: [
                {
                    data: dataPerMinute.map(item => item.total_logs),
                    type: 'bar'
                }
            ]
        };
    }
    const optionsForDataPerMinuteGroupByTaskId = () => {
        if (!dataPerMinuteGroupByTaskId || !dataPerMinuteGroupByTaskId.all_minutes || !dataPerMinuteGroupByTaskId.list || dataPerMinuteGroupByTaskId.list.length === 0) {
            return {

                xAxis: {
                    type: 'category',
                    data: []  // Empty x-axis data
                },
                yAxis: {
                    name: '同步次数',
                    type: 'value',
                },
                series: []  // No series data
            };
        }


        return {
            title: {
                text: '同步任务今日同步总览'

            },
            tooltip: {
                trigger: 'axis',
                axisPointer: {
                    type: 'cross',
                    label: {
                        backgroundColor: '#6a7985'
                    }
                }
            },
            legend: {
                data: dataPerMinuteGroupByTaskId.list.map(task => task.sync_task_name),
            },

            grid: {
                left: '3%',
                right: '4%',
                bottom: '3%',
                containLabel: true
            },
            xAxis: [
                {
                    type: 'category',
                    boundaryGap: false,
                    data: dataPerMinuteGroupByTaskId.all_minutes
                }
            ],
            yAxis: [
                {
                    type: 'value'
                }
            ],
            series: dataPerMinuteGroupByTaskId?.list.map(task => ({
                name: task.sync_task_name,  // Sync task name for each series
                data: task.total_logs,      // Corresponding logs data
                type: 'line',                // Type of chart (bar in this case)
                emphasis: {
                    focus: 'series'
                },
                stack: 'Total',

            }))
        };
    }
    const optionsForDataPerDay = () => {


        return {
            title: {
                left: 'center',
                text: '最近一个月同步总览'
            },
            xAxis: {
                type: 'category',
                data: datePerDay.map(item => item.day)
            },
            yAxis: {
                name: '同步次数',

                type: 'value',

            },
            series: [
                {
                    data: datePerDay.map(item => item.total_logs),
                    type: 'bar'
                }
            ]
        };
    }


    return (
        <div className="flex flex-col">
            <Card className="m-10 basis-12 	">
                <div className="flex flex-col overflow-auto	">
                    <h5 className="text-2xl font-bold tracking-tight text-gray-900 dark:text-white basis-12 text-center">
                        Tuna数据同步平台
                        <div className="flex flex-row">
                            <div className="basis-1/2">
                                <ReactECharts option={optionsForDataPerMinute()} />

                            </div>
                            <div className="basis-1/2">
                                <ReactECharts option={optionsForDataPerDay()} />

                            </div>
                        </div>
                        <div className="flex flex-row">
                            <div className="basis-1/2">
                                <ReactECharts option={optionsForDataPerMinuteGroupByTaskId()} />

                            </div>
                            <div className="basis-1/2">
                                <ReactECharts option={optionsForDataPerDay()} />

                            </div>
                        </div>
                    </h5>

                </div>
            </Card >

        </div >
    );
}
export default Dashboard;