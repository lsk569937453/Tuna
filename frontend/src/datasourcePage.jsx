import { Button, Card, Pagination } from "flowbite-react";
import { Table } from "flowbite-react";
import { HiOutlinePlus, HiSearch, HiShoppingCart } from "react-icons/hi";
import ReactECharts from 'echarts-for-react';
import Request from "./utils/axiosUtils";
import { useEffect, useState } from "react"; import BatteryGauge from 'react-battery-gauge'
import moment from 'moment';


const pageSize = 5;

function DatasourcePage() {


    const [chart1Data, setChart1Data] = useState([]);



    const getFurnaceData = () => {
        Request.get("/api/getFurnaceInfor").then((res) => {
            console.log(res);
            const mesArray = res.data.message.map(
                ({
                    name: name,
                    status: status,
                    weight: weight

                }) => {
                    return {
                        name,
                        status,
                        weight
                    };
                }
            );
            const tdata = mesArray.slice(0, pageSize);

            setTableData(tdata);
            setAllTableData(mesArray);
            setTotalTablePage(Math.ceil(mesArray.length / pageSize));
        });
    };


    return (
        <div className="flex flex-col">
            <div className="p-4 flex-col">
                <div className="mb-4 flex justify-center">
                    <Button >添加数据源</Button>
                </div>
                <Table>
                    <Table.Head>
                        <Table.HeadCell className="font-bold text-center text-xl">数据源名称</Table.HeadCell>
                        <Table.HeadCell className="font-bold text-center text-xl">数据源地址</Table.HeadCell>
                        <Table.HeadCell className="font-bold text-center text-xl">操作</Table.HeadCell>
                    </Table.Head>
                    <Table.Body className="divide-y">
                        <Table.Row className="bg-white dark:border-gray-700 dark:bg-gray-800">
                            <Table.Cell className="whitespace-nowrap font-medium text-gray-900 dark:text-white text-center">
                                {'Apple MacBook Pro 17"'}
                            </Table.Cell>
                            <Table.Cell className="text-center">Sliver</Table.Cell>

                            <Table.Cell className="text-center">
                                <a href="#" className="font-medium text-cyan-600 hover:underline dark:text-cyan-500">
                                    删除
                                </a>
                            </Table.Cell>
                        </Table.Row>
                    </Table.Body>
                </Table>
            </div>
        </div >
    );
}
export default DatasourcePage;