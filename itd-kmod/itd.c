// SPDX-License-Identifier: MIT
// Copyright (C) 2023 Akira Moroo

#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/module.h>
#include <linux/cpumask.h>
#include <asm/msr.h>

MODULE_DESCRIPTION("Intel Thread Director (ITD) driver");
MODULE_AUTHOR("Akira Moroo <retrage01@gmail.com>");
MODULE_LICENSE("MIT");

static const u32 IA32_HW_FEEDBACK_THREAD_CONFIG = 0x17D4;

static int itd_init(void)
{
    int cpu = 0;
    for_each_possible_cpu(cpu) {
        u64 val = 0;
        rdmsrl_on_cpu(cpu, IA32_HW_FEEDBACK_THREAD_CONFIG, &val);
        if ((val & 0x1) == 0x0) {
            val |= 0x1;
            wrmsrl_on_cpu(cpu, IA32_HW_FEEDBACK_THREAD_CONFIG, val);
        }
    }

    return 0;
}

static void itd_exit(void)
{
    int cpu = 0;
    for_each_possible_cpu(cpu) {
        u64 val = 0;
        rdmsrl_on_cpu(cpu, IA32_HW_FEEDBACK_THREAD_CONFIG, &val);
        if ((val & 0x1) == 0x1) {
            val &= ~0x1;
            wrmsrl_on_cpu(cpu, IA32_HW_FEEDBACK_THREAD_CONFIG, val);
        }
    }
}

module_init(itd_init);
module_exit(itd_exit);
